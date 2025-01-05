use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum Rental {
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
                    .table(Rental::Table)
                    .if_not_exists()
                    .col(pk_auto(Rental::Id))
                    .col(integer_uniq(Rental::GameId))
                    .col(integer(Rental::UserId))
                    .col(date(Rental::RentalDate))
                    .col(date(Rental::ReturnDate))
                    .col(boolean(Rental::PickedUp).default(false))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_rental_game")
                            .from(Rental::Table, Rental::GameId)
                            .to(BoardGame::Table, BoardGame::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_rental_user")
                            .from(Rental::Table, Rental::UserId)
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
            .drop_table(Table::drop().table(Rental::Table).to_owned())
            .await
    }
}
