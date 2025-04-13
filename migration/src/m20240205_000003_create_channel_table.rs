use sea_orm_migration::prelude::*;

use crate::m20240205_000002_create_guild_table::Guild;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Channel::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Channel::Id)
                            .big_unsigned()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Channel::Name).string().not_null())
                    .col(ColumnDef::new(Channel::GuildId).big_unsigned().not_null())
                    .col(ColumnDef::new(Channel::LastMessageId).big_unsigned())
                    .col(
                        ColumnDef::new(Channel::BackfillDone)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Channel::Table, Channel::GuildId)
                            .to(Guild::Table, Guild::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Channel::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Channel {
    Table,
    Id,
    Name,
    GuildId,
    LastMessageId,
    BackfillDone,
}
