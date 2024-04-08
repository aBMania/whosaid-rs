use std::time::Duration;

use anyhow::Result;
use sea_orm::{ConnectOptions, Database as SeaOrmDatabase, DatabaseConnection};

use crate::utils::workspace_dir;

pub(crate) mod user;
pub(crate) mod error;
mod guild;
mod channel;
mod messages;

pub struct Database {
    db: DatabaseConnection,
}


impl Database {
    pub async fn new() -> Result<Self> {
        let db_url = "postgres://postgres:whosaidp@localhost:15432/whosaid";

        let mut opt = ConnectOptions::new(db_url);

        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .acquire_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(true)
            .sqlx_logging_level(log::LevelFilter::Info)
            .set_schema_search_path("whosaid"); // Setting default PostgreSQL schema

        let db = SeaOrmDatabase::connect(db_url).await?;


        migration::migrate(&db).await.expect("migration failed");

        Ok(Self {
            db
        })
    }
}
