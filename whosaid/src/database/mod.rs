use std::time::Duration;

use anyhow::Result;
use sea_orm::{ConnectOptions, Database as SeaOrmDatabase, DatabaseConnection, DbErr};

use crate::utils::workspace_dir;

mod user;
mod error;
mod guild;
mod channel;
mod messages;

pub struct Database {
    db: DatabaseConnection,
}


#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("database error: {0}")]
    DbError(#[from] DbErr),

    #[error("Not found")]
    NotFound
}

impl Database {
    pub async fn new() -> Result<Self> {

        let mut db_path = workspace_dir();
        db_path.push("database.sqlite?mode=rwc");

        let db_path = db_path.to_str().unwrap();
        let db_url = format!("sqlite://{db_path}");

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

        let db = SeaOrmDatabase::connect(opt).await?;


        migration::migrate(&db).await.expect("migration failed");

        Ok(Self {
            db
        })
    }
}
