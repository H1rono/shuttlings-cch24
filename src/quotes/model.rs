use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[must_use]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, sqlx::Type,
)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct QuoteId(pub Uuid);

impl QuoteId {
    pub const COLUMN_NAME: &str = "id";
}

#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct Author(pub String);

impl Author {
    pub const COLUMN_NAME: &str = "author";
}

#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct QuoteText(pub String);

impl QuoteText {
    pub const COLUMN_NAME: &str = "quote";
}

#[must_use]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, sqlx::Type,
)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct CreatedAt(pub DateTime<Utc>);

impl CreatedAt {
    pub const COLUMN_NAME: &str = "created_at";
}

#[must_use]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, sqlx::Type,
)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct Version(pub i32);

impl Version {
    pub const COLUMN_NAME: &str = "version";
}

impl Default for Version {
    fn default() -> Self {
        Self(1)
    }
}

#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, sqlx::FromRow)]
pub struct Quote {
    pub id: QuoteId,
    pub author: Author,
    pub quote: QuoteText,
    pub created_at: CreatedAt,
    pub version: Version,
}

impl Quote {
    pub const TABLE_NAME: &str = "quotes";
}
