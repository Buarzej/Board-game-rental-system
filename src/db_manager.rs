use entity::prelude::{BoardGame, User, Rental, RentalHistory, ExtensionRequest};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, DbErr, EntityTrait, ModelTrait};
use entity::board_game::{ActiveModel as BoardGameActiveModel, Model as BoardGameModel};
use entity::user::{ActiveModel as UserActiveModel, Model as UserModel};
use entity::rental::{ActiveModel as RentalActiveModel, Model as RentalModel};
use entity::rental_history::{ActiveModel as RentalHistoryActiveModel, Model as RentalHistoryModel};
use entity::extension_request::{ActiveModel as ExtensionRequestActiveModel, Model as ExtensionRequestModel};

const DB_NAME: &str = "database";
const DATABASE_URL: &str = "sqlite:./database.db?mode=rwc";
const PENALTY_THRESHOLD: u8 = 2;

pub async fn initialize_database() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(DATABASE_URL).await?;
    Migrator::up(&db, None).await?;

    Ok(db)
}

pub async fn save_board_game(
    board_game: BoardGameActiveModel,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    board_game.save(db).await?;
    Ok(())
}

pub async fn get_board_game(id: i32, db: &DatabaseConnection) -> Result<Option<BoardGameModel>, DbErr> {
    let board_game = BoardGame::find_by_id(id).one(db).await?;
    Ok(board_game)
}

pub async fn delete_board_game(id: i32, db: &DatabaseConnection) -> Result<(), DbErr> {
    BoardGame::delete_by_id(id).exec(db).await?;
    Ok(())
}

pub async fn save_user(user: UserActiveModel, db: &DatabaseConnection) -> Result<(), DbErr> {
    user.save(db).await?;
    Ok(())
}

pub async fn get_user(id: i32, db: &DatabaseConnection) -> Result<Option<UserModel>, DbErr> {
    let user = User::find_by_id(id).one(db).await?;
    Ok(user)
}

pub async fn is_user_penalized(id: i32, db: &DatabaseConnection) -> Result<bool, DbErr> {
    let user = User::find_by_id(id).one(db).await?;
    Ok(user.map_or(false, |u| u.penalty_points > PENALTY_THRESHOLD))
}

pub async fn delete_user(id: i32, db: &DatabaseConnection) -> Result<(), DbErr> {
    User::delete_by_id(id).exec(db).await?;
    Ok(())
}

pub async fn save_rental(rental: RentalActiveModel, db: &DatabaseConnection) -> Result<(), DbErr> {
    rental.save(db).await?;
    Ok(())
}

pub async fn get_rental(id: i32, db: &DatabaseConnection) -> Result<Option<RentalModel>, DbErr> {
    let rental = Rental::find_by_id(id).one(db).await?;
    Ok(rental)
}

pub async fn delete_rental(id: i32, db: &DatabaseConnection) -> Result<(), DbErr> {
    Rental::delete_by_id(id).exec(db).await?;
    Ok(())
}