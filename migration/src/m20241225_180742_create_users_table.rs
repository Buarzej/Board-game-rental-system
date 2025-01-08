use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id,
    Name,
    Surname,
    Email,
    PasswordHash,
    PenaltyPoints,
    IsAdmin,
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
                    .col(string_uniq(User::Email))
                    .col(string(User::PasswordHash))
                    .col(tiny_unsigned(User::PenaltyPoints).default(0))
                    .col(boolean(User::IsAdmin).default(false))
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
