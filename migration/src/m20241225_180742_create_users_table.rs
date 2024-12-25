use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id,
    Name,
    Surname,
    Phone,
    Email,
    Password,
    PenaltyPoints,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(integer(User::Id).primary_key())
                    .col(string(User::Name))
                    .col(string(User::Surname))
                    .col(string(User::Phone))
                    .col(string(User::Email))
                    .col(string(User::Password))
                    .col(tiny_unsigned(User::PenaltyPoints).default(0))
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
