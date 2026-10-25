use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(User::Id)
                            .big_unsigned()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(User::Name).string().not_null())
                    // integer, as created in production: sea-query 1 maps unsigned() to bigint on Postgres
                    .col(ColumnDef::new(User::Discriminator).integer())
                    .col(ColumnDef::new(User::GlobalName).string())
                    .col(ColumnDef::new(User::Bot).boolean())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id,
    Name,
    Discriminator,
    GlobalName,
    Bot,
}
