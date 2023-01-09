use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub mod pagination;
pub mod repository;
pub mod subscribe;
pub mod user;

pub use sqlx::postgres::PgPoolOptions;

#[async_trait]
pub trait Insert {
    type Output;
    async fn insert(self, db: &PgPool) -> sqlx::Result<Self::Output>;
}
