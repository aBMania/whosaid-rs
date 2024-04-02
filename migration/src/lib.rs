use std::error::Error;
pub use sea_orm_migration::prelude::*;
use crate::sea_orm::DbConn;

mod m20240205_000001_create_user_table;
mod m20240205_000002_create_guild_table;
mod m20240205_000003_create_channel_table;
mod m20240205_000004_create_message_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240205_000001_create_user_table::Migration),
            Box::new(m20240205_000002_create_guild_table::Migration),
            Box::new(m20240205_000003_create_channel_table::Migration),
            Box::new(m20240205_000004_create_message_table::Migration),
        ]
    }
}

pub async fn migrate(
    db: &DbConn,
) -> Result<(), Box<dyn Error>> {
    Migrator::up(db, None).await?;
    Ok(())
}