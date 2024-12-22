use serde::{Deserialize, Serialize};
use uuid::Uuid;
use warp::http::StatusCode;

use crate::quotes;

pub struct State {
    pub(super) repository: quotes::Repository,
}

pub struct Builder<Repository = ()> {
    repository: Repository,
}

impl Default for Builder<()> {
    fn default() -> Self {
        Self { repository: () }
    }
}

impl<Repository> Builder<Repository> {
    pub fn repository(self, value: quotes::Repository) -> Builder<quotes::Repository> {
        Builder { repository: value }
    }
}

impl State {
    pub fn builder() -> Builder<()> {
        Builder::default()
    }
}

impl Builder<quotes::Repository> {
    pub fn build(self) -> State {
        let Self { repository } = self;
        State { repository }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub(crate) struct CitePathParam {
    pub(super) id: quotes::model::QuoteId,
}

impl CitePathParam {
    pub(crate) fn new(id: Uuid) -> Self {
        Self {
            id: quotes::model::QuoteId(id),
        }
    }
}

pub(super) async fn find_and_serialize_cite(
    state: &State,
    param: CitePathParam,
) -> Result<String, StatusCode> {
    let CitePathParam { id } = param;
    let quote = state
        .repository
        .find_one(id)
        .await
        .map_err(|e| {
            tracing::error!(err = &e as &dyn std::error::Error, "Failed to cite");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!("Found no quote");
            StatusCode::NOT_FOUND
        })?;
    tracing::info!("Found one quote");
    serde_json::to_string(&quote).map_err(|e| {
        tracing::error!(
            err = &e as &dyn std::error::Error,
            ?quote,
            "Failed to serialize JSON"
        );
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub(crate) struct RemovePathParam {
    pub(super) id: quotes::model::QuoteId,
}

impl RemovePathParam {
    pub(crate) fn new(id: Uuid) -> Self {
        Self {
            id: quotes::model::QuoteId(id),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub(crate) struct UndoPathParam {
    pub(super) id: quotes::model::QuoteId,
}

impl UndoPathParam {
    pub(crate) fn new(id: Uuid) -> Self {
        Self {
            id: quotes::model::QuoteId(id),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UndoBody {
    pub(super) author: quotes::model::Author,
    pub(super) quote: quotes::model::QuoteText,
}

impl State {
    pub(super) async fn undo_aka_update(
        &self,
        param: UndoPathParam,
        body: UndoBody,
    ) -> sqlx::Result<Option<quotes::model::Quote>> {
        let UndoPathParam { id } = param;
        let UndoBody { author, quote } = body;
        let request = quotes::ops::UpdateRequest { id, author, quote };
        self.repository.update_one(request).await
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DraftBody {
    pub(super) author: quotes::model::Author,
    pub(super) quote: quotes::model::QuoteText,
}

impl From<DraftBody> for quotes::ops::CreateRequest {
    fn from(value: DraftBody) -> Self {
        let DraftBody { author, quote } = value;
        Self { author, quote }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct ListQuery {
    pub(super) token: Option<String>,
}
