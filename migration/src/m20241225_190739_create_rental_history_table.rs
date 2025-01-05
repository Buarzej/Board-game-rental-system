use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum RentalHistory {
    Table,
    Id,
    GameId,
    UserId,
    RentalDate,
    ReturnDate,
    PickedUp,
}

#[derive(DeriveIden)]
pub enum BoardGame {
    Table,
    Id,
}

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RentalHistory::Table)
                    .if_not_exists()
                    .col(integer(RentalHistory::Id).primary_key())
                    .col(integer(RentalHistory::GameId))
                    .col(integer(RentalHistory::UserId))
                    .col(date(RentalHistory::RentalDate))
                    .col(date(RentalHistory::ReturnDate))
                    .col(boolean(RentalHistory::PickedUp))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_rental_history_game")
                            .from(RentalHistory::Table, RentalHistory::GameId)
                            .to(BoardGame::Table, BoardGame::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_rental_history_user")
                            .from(RentalHistory::Table, RentalHistory::UserId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RentalHistory::Table).to_owned())
            .await
    }
}
