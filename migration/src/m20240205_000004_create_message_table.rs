use sea_orm_migration::prelude::*;
use crate::m20240205_000001_create_user_table::User;

use crate::m20240205_000003_create_channel_table::Channel;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Message::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Message::Id)
                            .big_unsigned()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Message::ChannelId)
                        .big_unsigned()
                        .not_null()
                    )
                    .col(ColumnDef::new(Message::AuthorId)
                        .big_unsigned()
                        .not_null()
                    )
                    .col(ColumnDef::new(Message::Content)
                        .text()
                        .not_null()
                    )
                    .col(ColumnDef::new(Message::Timestamp)
                        .timestamp_with_time_zone()
                        .not_null()
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Message::Table, Message::ChannelId)
                            .to(Channel::Table, Channel::Id)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Message::Table, Message::AuthorId)
                            .to(User::Table, User::Id)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Message::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Message {
    Table,
    Id,
    AuthorId,
    ChannelId,
    Content,
    Timestamp,
}
