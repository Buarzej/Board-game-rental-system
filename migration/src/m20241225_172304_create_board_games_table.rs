use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum BoardGame {
    Table,
    Id,
    Title,
    Weight,
    PhotoFilename,
    MinPlayers,
    MaxPlayers,
    MinPlaytime,
    MaxPlaytime,
    AdditionalInfo,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BoardGame::Table)
                    .if_not_exists()
                    .col(pk_auto(BoardGame::Id))
                    .col(string(BoardGame::Title))
                    .col(small_unsigned(BoardGame::Weight))
                    .col(string(BoardGame::PhotoFilename))
                    .col(tiny_unsigned(BoardGame::MinPlayers))
                    .col(tiny_unsigned(BoardGame::MaxPlayers))
                    .col(small_unsigned(BoardGame::MinPlaytime))
                    .col(small_unsigned(BoardGame::MaxPlaytime))
                    .col(text_null(BoardGame::AdditionalInfo))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BoardGame::Table).to_owned())
            .await
    }
}
