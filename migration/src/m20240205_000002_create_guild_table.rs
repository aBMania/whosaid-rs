use super::m20240205_000001_create_user_table::User;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Guild::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Guild::Id)
                            .big_unsigned()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Guild::Name).string().not_null())
                    .col(ColumnDef::new(Guild::OwnerId).big_unsigned().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Guild::Table, Guild::OwnerId)
                            .to(User::Table, User::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Guild::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Guild {
    Table,
    Id,
    Name,
    OwnerId,
}
