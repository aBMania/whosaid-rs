use sea_orm_migration::prelude::*;

use crate::m20240205_000004_create_message_table::Message;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Message::Table)
                    .modify_column(ColumnDef::new(Message::AuthorId).null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Message::Table)
                    .modify_column(ColumnDef::new(Message::AuthorId).not_null())
                    .to_owned(),
            )
            .await
    }
}
