use serde::{Deserialize, Serialize};
use sqlx::migrate::{MigrateError, Migrator};
use uuid::Uuid;

use super::model::{Author, Quote, QuoteId, QuoteText, Version};
use super::Repository;

pub static MIGRATOR: Migrator = sqlx::migrate!("src/quotes/migrations");

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

impl Repository {
    #[tracing::instrument(skip_all)]
    pub async fn migrate(&self) -> Result<(), MigrateError> {
        MIGRATOR.run(&self.inner.pool).await
    }

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
            r#"SELECT * FROM "{}" WHERE "{}" = ? LIMIT 1"#,
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
            r#"INSERT INTO "{}" ("{}", "{}", "{}") VALUES (?, ?, ?) RETURNING *"#,
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
    pub async fn delete_one(&self, id: QuoteId) -> sqlx::Result<Option<()>> {
        let query = format!(
            r#"DELETE FROM "{}" WHERE "{}" = ? LIMIT 1"#,
            Quote::TABLE_NAME,
            QuoteId::COLUMN_NAME
        );
        let quote: Option<Quote> = sqlx::query_as(&query)
            .bind(id)
            .fetch_optional(&self.inner.pool)
            .await?;
        Ok(quote.map(|_| ()))
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
                SET "{author}" = ?, "{quote}" = ?, "{version}" = "{version}" + 1
                WHERE "{id}" = ?
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
}
