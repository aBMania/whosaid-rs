use sea_orm_migration::prelude::*;

use crate::m20240205_000002_create_guild_table::Guild;
use crate::m20240205_000001_create_user_table::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {

    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .primary_key(Index::create().col(UserEmoji::UserId).col(UserEmoji::GuildId))
                    .table(UserEmoji::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(UserEmoji::UserId)
                        .big_unsigned()
                        .not_null()
                    )
                    .col(ColumnDef::new(UserEmoji::GuildId)
                        .big_unsigned()
                        .not_null()
                    )
                    .col(ColumnDef::new(UserEmoji::Emoji)
                        .string()
                        .not_null()
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserEmoji::Table, UserEmoji::UserId)
                            .to(User::Table, Guild::Id)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserEmoji::Table, UserEmoji::GuildId)
                            .to(Guild::Table, Guild::Id)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserEmoji::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum UserEmoji {
    Table,
    UserId,
    GuildId,
    Emoji,
}
