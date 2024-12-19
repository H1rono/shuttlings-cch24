use std::sync::Arc;

use sqlx::PgPool;

#[derive(Clone)]
pub(super) struct Inner {
    pub(super) pool: PgPool,
}

#[derive(Clone)]
pub struct Builder<Pool = ()> {
    pool: Pool,
}

impl Default for Builder<()> {
    fn default() -> Self {
        Self { pool: () }
    }
}

impl<Pool> Builder<Pool> {
    pub fn pool(self, pool: PgPool) -> Builder<PgPool> {
        Builder { pool }
    }
}

impl Builder<PgPool> {
    pub fn build(self) -> super::Repository {
        let Self { pool } = self;
        let inner = Inner { pool };
        super::Repository {
            inner: Arc::new(inner),
        }
    }
}

impl super::Repository {
    pub fn builder() -> Builder {
        Builder::default()
    }
}
