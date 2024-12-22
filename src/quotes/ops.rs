use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::model::{Author, CreatedAt, Quote, QuoteId, QuoteText, Version};
use super::{shorten, Repository, TokenError};

#[must_use]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateRequest {
    pub author: Author,
    pub quote: QuoteText,
}

#[must_use]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateRequest {
    pub id: QuoteId,
    pub author: Author,
    pub quote: QuoteText,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListResponse {
    pub quotes: Vec<Quote>,
    pub page: u64,
    pub next_token: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ListError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Token(#[from] TokenError),
}

impl Repository {
    #[tracing::instrument(skip_all)]
    pub async fn reset(&self) -> sqlx::Result<()> {
        let query = format!(r#"DELETE FROM "{}""#, Quote::TABLE_NAME);
        let res = sqlx::query(&query).execute(&self.inner.pool).await?;
        tracing::info!(rows_affected = res.rows_affected());
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub async fn find_one(&self, id: QuoteId) -> sqlx::Result<Option<Quote>> {
        let query = format!(
            r#"SELECT * FROM "{}" WHERE "{}" = $1 LIMIT 1"#,
            Quote::TABLE_NAME,
            QuoteId::COLUMN_NAME
        );
        let quote: Option<Quote> = sqlx::query_as(&query)
            .bind(id)
            .fetch_optional(&self.inner.pool)
            .await?;
        tracing::info!("SELECTed a quote");
        Ok(quote)
    }

    #[tracing::instrument(skip_all)]
    pub async fn create(&self, request: CreateRequest) -> sqlx::Result<Quote> {
        let CreateRequest { author, quote } = request;
        let id = QuoteId(Uuid::new_v4());
        let query = format!(
            r#"INSERT INTO "{}" ("{}", "{}", "{}") VALUES ($1, $2, $3) RETURNING *"#,
            Quote::TABLE_NAME,
            QuoteId::COLUMN_NAME,
            Author::COLUMN_NAME,
            QuoteText::COLUMN_NAME,
        );
        let quote: Quote = sqlx::query_as(&query)
            .bind(id)
            .bind(author)
            .bind(quote)
            .fetch_one(&self.inner.pool)
            .await?;
        tracing::info!("INSERTed a quote");
        Ok(quote)
    }

    #[tracing::instrument(skip_all)]
    pub async fn delete_one(&self, id: QuoteId) -> sqlx::Result<Option<Quote>> {
        let query = format!(
            r#"DELETE FROM "{}" WHERE "{}" = $1 RETURNING *"#,
            Quote::TABLE_NAME,
            QuoteId::COLUMN_NAME
        );
        let quote: Option<Quote> = sqlx::query_as(&query)
            .bind(id)
            .fetch_optional(&self.inner.pool)
            .await?;
        Ok(quote)
    }

    #[tracing::instrument(skip_all)]
    pub async fn update_one(&self, request: UpdateRequest) -> sqlx::Result<Option<Quote>> {
        let UpdateRequest {
            id,
            author,
            quote: quote_text,
        } = request;
        let query = format!(
            r#"
                UPDATE "{table}"
                SET "{author}" = $1, "{quote}" = $2, "{version}" = "{version}" + 1
                WHERE "{id}" = $3
                RETURNING *
            "#,
            table = Quote::TABLE_NAME,
            id = QuoteId::COLUMN_NAME,
            author = Author::COLUMN_NAME,
            quote = QuoteText::COLUMN_NAME,
            version = Version::COLUMN_NAME,
        );
        let quote: Option<Quote> = sqlx::query_as(&query)
            .bind(author)
            .bind(quote_text)
            .bind(id)
            .fetch_optional(&self.inner.pool)
            .await?;
        tracing::info!("UPDATEd a quote");
        Ok(quote)
    }

    pub async fn list(&self, next_token: Option<&str>) -> Result<Option<ListResponse>, ListError> {
        if let Some(next_token) = next_token {
            self.list_non_first(next_token).await
        } else {
            self.list_first().await
        }
    }

    async fn list_first(&self) -> Result<Option<ListResponse>, ListError> {
        let query = format!(
            // FIXME: custom number of quotes per page
            r#"SELECT * FROM "{}" ORDER BY "{}" ASC LIMIT 4"#,
            Quote::TABLE_NAME,
            CreatedAt::COLUMN_NAME
        );
        let quotes = sqlx::query_as(&query).fetch_all(&self.inner.pool);
        let (quotes, next_token) = self.list_split_next(quotes).await?;
        let res = ListResponse {
            quotes,
            page: 1,
            next_token,
        };
        Ok(Some(res))
    }

    async fn list_non_first(&self, next_token: &str) -> Result<Option<ListResponse>, ListError> {
        let next_token = shorten::Token::parse_alphanumeric(next_token)?;
        let query = format!(
            r#"
                SELECT * FROM "{}"
                WHERE replace("{}"::text, '-', '') LIKE $1 || '%'
                LIMIT 1
            "#,
            Quote::TABLE_NAME,
            QuoteId::COLUMN_NAME,
        );
        let head: Option<Quote> = sqlx::query_as(&query)
            .bind(next_token.as_hex().to_string())
            .fetch_optional(&self.inner.pool)
            .await?;
        let Some(head) = head else {
            return Ok(None);
        };
        let query = format!(
            r#"
                SELECT * FROM "{table}"
                WHERE "{created_at}" >= $1
                ORDER BY "{created_at}" ASC LIMIT 4
            "#,
            table = Quote::TABLE_NAME,
            created_at = CreatedAt::COLUMN_NAME
        );
        let quotes = sqlx::query_as(&query)
            .bind(head.created_at)
            .fetch_all(&self.inner.pool);
        let (quotes, next_token) = self.list_split_next(quotes).await?;
        let page = self.page_number_of(&head).await?;
        let res = ListResponse {
            quotes,
            page,
            next_token,
        };
        Ok(Some(res))
    }

    async fn list_split_next<F>(&self, fut: F) -> sqlx::Result<(Vec<Quote>, Option<String>)>
    where
        F: std::future::IntoFuture<Output = sqlx::Result<Vec<Quote>>>,
    {
        let mut quotes = fut.await?;
        let next_token = quotes
            .get(3)
            .map(|q| shorten::Token::new(q.id.0).as_alphanumeric().to_string());
        quotes.truncate(3);
        Ok((quotes, next_token))
    }

    async fn page_number_of(&self, head: &Quote) -> sqlx::Result<u64> {
        #[derive(sqlx::FromRow)]
        struct Row {
            count: i64,
        }

        let query = format!(
            r#"SELECT COUNT(*) AS "count" FROM "{}" WHERE "{}" < $1"#,
            Quote::TABLE_NAME,
            CreatedAt::COLUMN_NAME,
        );
        let res: Row = sqlx::query_as(&query)
            .bind(head.created_at)
            .fetch_one(&self.inner.pool)
            .await?;
        Ok(res.count as u64 / 3 + 1)
    }
}
