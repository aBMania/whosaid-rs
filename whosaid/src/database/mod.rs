use std::env;
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

        let db_url = env::var("DATABASE_URL")?;

        let db = SeaOrmDatabase::connect(db_url).await?;


        migration::migrate(&db).await.expect("migration failed");

        Ok(Self {
            db
        })
    }
}
