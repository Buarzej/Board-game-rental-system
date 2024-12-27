use super::m20241225_172304_create_board_games_table::BoardGame;
use super::m20241225_180742_create_users_table::User;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Favourite {
    Table,
    UserId,
    GameId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Favourite::Table)
                    .if_not_exists()
                    .col(integer(Favourite::UserId))
                    .col(integer(Favourite::GameId))
                    .primary_key(
                        Index::create()
                            .name("pk_favourite")
                            .col(Favourite::UserId)
                            .col(Favourite::GameId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_favourite_user")
                            .from(Favourite::Table, Favourite::UserId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_favourite_game")
                            .from(Favourite::Table, Favourite::GameId)
                            .to(BoardGame::Table, BoardGame::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Favourite::Table).to_owned())
            .await
    }
}
