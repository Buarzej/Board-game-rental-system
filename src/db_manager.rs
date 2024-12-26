use entity::board_game::{ActiveModel as BoardGameActiveModel, Model as BoardGameModel};
use entity::extension_request::{
    ActiveModel as ExtensionRequestActiveModel, Model as ExtensionRequestModel,
};
use entity::favourite::{ActiveModel as FavouriteActiveModel, Model as FavouriteModel};
use entity::prelude::{BoardGame, ExtensionRequest, Favourite, Rental, RentalHistory, User};
use entity::rental::{ActiveModel as RentalActiveModel, Model as RentalModel};
use entity::rental_history::{ActiveModel as RentalHistoryActiveModel, Model as RentalHistoryModel};
use entity::user::{ActiveModel as UserActiveModel, Model as UserModel};
use entity::{board_game, user};
use migration::{Migrator, MigratorTrait};
use sea_orm::{
    ActiveModelTrait, ActiveValue, Database, DatabaseConnection, DbErr,
    EntityTrait, QueryOrder,
};

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

pub async fn get_board_game(
    id: i32,
    db: &DatabaseConnection,
) -> Result<Option<BoardGameModel>, DbErr> {
    let board_game = BoardGame::find_by_id(id).one(db).await?;
    Ok(board_game)
}

pub async fn get_all_board_games_admin(
    db: &DatabaseConnection,
) -> Result<Vec<(BoardGameModel, Option<RentalModel>)>, DbErr> {
    let board_games = BoardGame::find()
        .order_by_asc(board_game::Column::Title)
        .find_also_related(Rental)
        .all(db)
        .await?;
    Ok(board_games)
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

pub async fn get_all_users(db: &DatabaseConnection) -> Result<Vec<UserModel>, DbErr> {
    let users = User::find()
        .order_by_asc(user::Column::Surname)
        .all(db)
        .await?;
    Ok(users)
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

pub async fn archive_rental(id: i32, db: &DatabaseConnection) -> Result<(), DbErr> {
    let rental = Rental::find_by_id(id).one(db).await?;
    if let Some(rental) = rental {
        let rental_history = RentalHistoryActiveModel {
            id: ActiveValue::Set(rental.id),
            game_id: ActiveValue::Set(rental.game_id),
            user_id: ActiveValue::Set(rental.user_id),
            rental_date: ActiveValue::Set(rental.rental_date),
            return_date: ActiveValue::Set(rental.return_date),
            picked_up: ActiveValue::Set(rental.picked_up),
        };
        rental_history.insert(db).await?;
        Rental::delete_by_id(id).exec(db).await?;
    }
    Ok(())
}

pub async fn delete_rental_history(id: i32, db: &DatabaseConnection) -> Result<(), DbErr> {
    RentalHistory::delete_by_id(id).exec(db).await?;
    Ok(())
}

pub async fn save_extension_request(
    extension_request: ExtensionRequestActiveModel,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    extension_request.save(db).await?;
    Ok(())
}

pub async fn get_extension_request(
    id: i32,
    db: &DatabaseConnection,
) -> Result<Option<ExtensionRequestModel>, DbErr> {
    let extension_request = ExtensionRequest::find_by_id(id).one(db).await?;
    Ok(extension_request)
}

pub async fn delete_extension_request(id: i32, db: &DatabaseConnection) -> Result<(), DbErr> {
    ExtensionRequest::delete_by_id(id).exec(db).await?;
    Ok(())
}

pub async fn save_favourite(
    favourite: FavouriteActiveModel,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    favourite.save(db).await?;
    Ok(())
}

pub async fn get_favourite(
    id: i32,
    db: &DatabaseConnection,
) -> Result<Option<FavouriteModel>, DbErr> {
    let favourite = Favourite::find_by_id(id).one(db).await?;
    Ok(favourite)
}

pub async fn delete_favourite(id: i32, db: &DatabaseConnection) -> Result<(), DbErr> {
    Favourite::delete_by_id(id).exec(db).await?;
    Ok(())
}
