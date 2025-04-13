use std::env;

use anyhow::Result;
use sea_orm::{Database as SeaOrmDatabase, DatabaseConnection};

mod channel;
pub(crate) mod error;
mod guild;
mod messages;
pub(crate) mod user;

pub struct Database {
    db: DatabaseConnection,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let db_url = env::var("DATABASE_URL")?;

        let db = SeaOrmDatabase::connect(db_url).await?;

        migration::migrate(&db).await.expect("migration failed");

        Ok(Self { db })
    }
}
