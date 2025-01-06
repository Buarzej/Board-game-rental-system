use entity::board_game::{ActiveModel as BoardGameActiveModel, Model as BoardGameModel};
use entity::extension_request::{
    ActiveModel as ExtensionRequestActiveModel, Model as ExtensionRequestModel,
};
use entity::favourite::ActiveModel as FavouriteActiveModel;
use entity::prelude::{BoardGame, ExtensionRequest, Favourite, Rental, RentalHistory, User};
use entity::rental::{ActiveModel as RentalActiveModel, Model as RentalModel};
use entity::rental_history::ActiveModel as RentalHistoryActiveModel;
use entity::user::{ActiveModel as UserActiveModel, Model as UserModel};
use entity::{board_game, extension_request, favourite, rental, rental_history, user};
use migration::{Expr, JoinType, Migrator, MigratorTrait};
use sea_orm::prelude::Date;
use sea_orm::sea_query::IntoCondition;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Database, DatabaseConnection, DbErr,
    EntityTrait, FromQueryResult, Iterable, QueryFilter, QueryOrder, QuerySelect,
    RelationTrait, TransactionTrait,
};

const DATABASE_URL: &str = "sqlite:./database.db?mode=rwc";
const PENALTY_THRESHOLD: u8 = 2;

/// Initializes the database connection and runs the necessary migrations.
pub async fn initialize_database() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(DATABASE_URL).await?;
    Migrator::up(&db, None).await?;

    Ok(db)
}

/// Saves a board game to the database. Handles both insertions and updates.
pub async fn save_board_game(
    board_game: BoardGameActiveModel,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    board_game.save(db).await?;
    Ok(())
}

/// Retrieves a board game of the given ID from the database.
pub async fn get_board_game(
    id: i32,
    db: &DatabaseConnection,
) -> Result<Option<BoardGameModel>, DbErr> {
    let board_game = BoardGame::find_by_id(id).one(db).await?;
    Ok(board_game)
}

#[derive(Debug, Eq, PartialEq, FromQueryResult)]
pub struct GetBoardGamesQueryResult {
    id: i32,
    title: String,
    photo_filename: String,
    min_players: u8,
    max_players: u8,
    min_playtime: u16,
    max_playtime: u16,
    return_date: Option<Date>,
    is_favourite: bool,
}

/// Retrieves all board games from the database, along with the information
/// about the rental status and whether they are in the user's favourites.
pub async fn get_board_games(
    user_id: i32,
    db: &DatabaseConnection,
) -> Result<Vec<GetBoardGamesQueryResult>, DbErr> {
    let board_games = BoardGame::find()
        .select_only()
        .columns(board_game::Column::iter().filter(|c| {
            !matches!(
                c,
                board_game::Column::Weight | board_game::Column::AdditionalInfo
            )
        }))
        .column(rental::Column::ReturnDate)
        .expr_as(
            Expr::case(
                Expr::col((favourite::Entity, favourite::Column::UserId)).is_not_null(),
                Expr::value(true),
            )
            .finally(Expr::value(false)),
            "is_favourite",
        )
        .join(
            JoinType::LeftJoin,
            board_game::Relation::Favourite
                .def()
                .on_condition(move |_left, right| {
                    Expr::col((right, favourite::Column::UserId))
                        .eq(user_id)
                        .into_condition()
                }),
        )
        .left_join(Rental)
        .order_by_asc(board_game::Column::Title)
        .into_model::<GetBoardGamesQueryResult>()
        .all(db)
        .await?;
    Ok(board_games)
}

/// Retrieves all board games from the database, along with their rental status.
/// Should be used by admin users only, as it provides additional info about rental.
pub async fn get_board_games_admin(
    db: &DatabaseConnection,
) -> Result<Vec<(BoardGameModel, Option<RentalModel>)>, DbErr> {
    let board_games = BoardGame::find()
        .order_by_asc(board_game::Column::Title)
        .find_also_related(Rental)
        .all(db)
        .await?;
    Ok(board_games)
}

/// Deletes a board game of the given ID from the database.
pub async fn delete_board_game(id: i32, db: &DatabaseConnection) -> Result<(), DbErr> {
    BoardGame::delete_by_id(id).exec(db).await?;
    Ok(())
}

/// Saves a user to the database. Handles both insertions and updates.
pub async fn save_user(user: UserActiveModel, db: &DatabaseConnection) -> Result<(), DbErr> {
    user.save(db).await?;
    Ok(())
}

/// Retrieves a user of the given ID from the database.
pub async fn get_user(id: i32, db: &DatabaseConnection) -> Result<Option<UserModel>, DbErr> {
    let user = User::find_by_id(id).one(db).await?;
    Ok(user)
}

/// Retrieves all users from the database.
pub async fn get_users(db: &DatabaseConnection) -> Result<Vec<UserModel>, DbErr> {
    let users = User::find()
        .order_by_asc(user::Column::Surname)
        .all(db)
        .await?;
    Ok(users)
}

/// Checks whether a user is penalized based on their penalty points.
/// A user is penalized if their penalty points exceed `PENALTY_THRESHOLD` const.
pub async fn is_user_penalized(id: i32, db: &DatabaseConnection) -> Result<bool, DbErr> {
    let user = User::find_by_id(id).one(db).await?;
    Ok(user.map_or(false, |u| u.penalty_points > PENALTY_THRESHOLD))
}

/// Deletes a user of the given ID from the database.
pub async fn delete_user(id: i32, db: &DatabaseConnection) -> Result<(), DbErr> {
    User::delete_by_id(id).exec(db).await?;
    Ok(())
}

/// Saves a rental to the database. Handles both insertions and updates.
pub async fn save_rental(rental: RentalActiveModel, db: &DatabaseConnection) -> Result<(), DbErr> {
    rental.save(db).await?;
    Ok(())
}

/// Retrieves a rental of the given ID from the database.
pub async fn get_rental(id: i32, db: &DatabaseConnection) -> Result<Option<RentalModel>, DbErr> {
    let rental = Rental::find_by_id(id).one(db).await?;
    Ok(rental)
}

/// Retrieves the rental of the given game ID from the database.
pub async fn get_game_rental_status(
    game_id: i32,
    db: &DatabaseConnection,
) -> Result<Option<RentalModel>, DbErr> {
    let rental = Rental::find()
        .filter(rental::Column::GameId.eq(game_id))
        .one(db)
        .await?;
    Ok(rental)
}

#[derive(Debug, Eq, PartialEq, FromQueryResult)]
pub struct GetRentalsQueryResult {
    id: i32,
    game_id: i32,
    user_id: i32,
    rental_date: Date,
    return_date: Date,
    picked_up: bool,
    title: String,
    photo_filename: String,
    name: String,
    surname: String,
    extension_date: Option<Date>,
}

/// Retrieves all rentals from the database, along with the information
/// about associated board games, users, and extension requests.
pub async fn get_rentals(db: &DatabaseConnection) -> Result<Vec<GetRentalsQueryResult>, DbErr> {
    let rentals = Rental::find()
        .columns([board_game::Column::Title, board_game::Column::PhotoFilename])
        .columns([user::Column::Name, user::Column::Surname])
        .column(extension_request::Column::ExtensionDate)
        .inner_join(BoardGame)
        .inner_join(User)
        .left_join(ExtensionRequest)
        .order_by_asc(rental::Column::RentalDate)
        .into_model::<GetRentalsQueryResult>()
        .all(db)
        .await?;
    Ok(rentals)
}

#[derive(Debug, Eq, PartialEq, FromQueryResult)]
pub struct GetUserRentalsQueryResult {
    id: i32,
    game_id: i32,
    rental_date: Date,
    return_date: Date,
    picked_up: bool,
    title: String,
    photo_filename: String,
    extension_date: Option<Date>,
    is_favorite: bool,
}

/// Retrieves all rentals from the database for the given user ID,
/// along with the information about associated board games,
/// extension requests, and whether they are in the user's favourites.
pub async fn get_user_rentals(
    user_id: i32,
    db: &DatabaseConnection,
) -> Result<Vec<GetUserRentalsQueryResult>, DbErr> {
    let user_rentals = Rental::find()
        .select_only()
        .columns(rental::Column::iter().filter(|c| !matches!(c, rental::Column::UserId)))
        .columns([board_game::Column::Title, board_game::Column::PhotoFilename])
        .column(extension_request::Column::ExtensionDate)
        .expr_as(
            Expr::case(
                Expr::col((favourite::Entity, favourite::Column::UserId)).is_not_null(),
                Expr::value(true),
            )
            .finally(Expr::value(false)),
            "is_favourite",
        )
        .inner_join(BoardGame)
        .left_join(ExtensionRequest)
        .join(
            JoinType::LeftJoin,
            board_game::Relation::Favourite
                .def()
                .on_condition(move |_left, right| {
                    Expr::col((right, favourite::Column::UserId))
                        .eq(user_id)
                        .into_condition()
                }),
        )
        .filter(rental::Column::UserId.eq(user_id))
        .order_by_asc(rental::Column::RentalDate)
        .into_model::<GetUserRentalsQueryResult>()
        .all(db)
        .await?;
    Ok(user_rentals)
}

#[derive(Debug, Eq, PartialEq, FromQueryResult)]
pub struct GetUserRentalsAdminQueryResult {
    id: i32,
    game_id: i32,
    rental_date: Date,
    return_date: Date,
    picked_up: bool,
    title: String,
    photo_filename: String,
    extension_date: Option<Date>,
}

/// Retrieves all rentals from the database for the given user ID,
/// along with the information about associated board games and extension requests.
/// Should be used by admin as it doesn't contain information about user favourites.
pub async fn get_user_rentals_admin(
    user_id: i32,
    db: &DatabaseConnection,
) -> Result<Vec<GetUserRentalsAdminQueryResult>, DbErr> {
    let user_rentals = Rental::find()
        .select_only()
        .columns(rental::Column::iter().filter(|c| !matches!(c, rental::Column::UserId)))
        .columns([board_game::Column::Title, board_game::Column::PhotoFilename])
        .column(extension_request::Column::ExtensionDate)
        .inner_join(BoardGame)
        .left_join(ExtensionRequest)
        .filter(rental::Column::UserId.eq(user_id))
        .order_by_asc(rental::Column::RentalDate)
        .into_model::<GetUserRentalsAdminQueryResult>()
        .all(db)
        .await?;
    Ok(user_rentals)
}

/// Deletes a rental of the given ID from the database.
pub async fn delete_rental(id: i32, db: &DatabaseConnection) -> Result<(), DbErr> {
    Rental::delete_by_id(id).exec(db).await?;
    Ok(())
}

/// Archives a rental of the given ID by moving it to the rental history table.
pub async fn archive_rental(id: i32, db: &DatabaseConnection) -> Result<(), DbErr> {
    let rental = Rental::find_by_id(id).one(db).await?;
    if let Some(rental) = rental {
        let txn = db.begin().await?;

        let rental_history = RentalHistoryActiveModel {
            id: ActiveValue::Set(rental.id),
            game_id: ActiveValue::Set(rental.game_id),
            user_id: ActiveValue::Set(rental.user_id),
            rental_date: ActiveValue::Set(rental.rental_date),
            return_date: ActiveValue::Set(rental.return_date),
            picked_up: ActiveValue::Set(rental.picked_up),
        };
        rental_history.insert(&txn).await?;
        Rental::delete_by_id(id).exec(&txn).await?;

        txn.commit().await?;
    }
    Ok(())
}

#[derive(Debug, Eq, PartialEq, FromQueryResult)]
pub struct GetRentalHistoryQueryResult {
    id: i32,
    game_id: i32,
    user_id: i32,
    rental_date: Date,
    return_date: Date,
    picked_up: bool,
    title: String,
    photo_filename: String,
    name: String,
    surname: String,
}

/// Retrieves all rental history entries from the database,
/// along with the information about associated board games and users.
pub async fn get_rental_history(
    db: &DatabaseConnection,
) -> Result<Vec<GetRentalHistoryQueryResult>, DbErr> {
    let rental_history = RentalHistory::find()
        .columns([board_game::Column::Title, board_game::Column::PhotoFilename])
        .columns([user::Column::Name, user::Column::Surname])
        .inner_join(BoardGame)
        .inner_join(User)
        .order_by_desc(rental_history::Column::ReturnDate)
        .into_model::<GetRentalHistoryQueryResult>()
        .all(db)
        .await?;
    Ok(rental_history)
}

#[derive(Debug, Eq, PartialEq, FromQueryResult)]
pub struct GetUserRentalHistoryQueryResult {
    id: i32,
    game_id: i32,
    rental_date: Date,
    return_date: Date,
    picked_up: bool,
    title: String,
    photo_filename: String,
    is_favorite: bool,
}

/// Retrieves all rental history entries from the database for the given user ID, along with
/// the information about associated board games and whether they are in the user's favourites.
pub async fn get_user_rental_history(
    user_id: i32,
    db: &DatabaseConnection,
) -> Result<Vec<GetUserRentalHistoryQueryResult>, DbErr> {
    let user_rental_history = Rental::find()
        .select_only()
        .columns(
            rental_history::Column::iter().filter(|c| !matches!(c, rental_history::Column::UserId)),
        )
        .columns([board_game::Column::Title, board_game::Column::PhotoFilename])
        .expr_as(
            Expr::case(
                Expr::col((favourite::Entity, favourite::Column::UserId)).is_not_null(),
                Expr::value(true),
            )
            .finally(Expr::value(false)),
            "is_favourite",
        )
        .inner_join(BoardGame)
        .join(
            JoinType::LeftJoin,
            board_game::Relation::Favourite
                .def()
                .on_condition(move |_left, right| {
                    Expr::col((right, favourite::Column::UserId))
                        .eq(user_id)
                        .into_condition()
                }),
        )
        .filter(rental_history::Column::UserId.eq(user_id))
        .order_by_desc(rental_history::Column::ReturnDate)
        .into_model::<GetUserRentalHistoryQueryResult>()
        .all(db)
        .await?;
    Ok(user_rental_history)
}

#[derive(Debug, Eq, PartialEq, FromQueryResult)]
pub struct GetUserRentalHistoryAdminQueryResult {
    id: i32,
    game_id: i32,
    rental_date: Date,
    return_date: Date,
    picked_up: bool,
    title: String,
    photo_filename: String,
}

/// Retrieves all rental history entries from the database for the given user ID,
/// along with the information about associated board games.
/// Should be used by admin users only, as it doesn't contain information about user favourites.
pub async fn get_user_rental_history_admin(
    user_id: i32,
    db: &DatabaseConnection,
) -> Result<Vec<GetUserRentalHistoryAdminQueryResult>, DbErr> {
    let user_rental_history = Rental::find()
        .select_only()
        .columns(
            rental_history::Column::iter().filter(|c| !matches!(c, rental_history::Column::UserId)),
        )
        .columns([board_game::Column::Title, board_game::Column::PhotoFilename])
        .inner_join(BoardGame)
        .filter(rental_history::Column::UserId.eq(user_id))
        .order_by_desc(rental_history::Column::ReturnDate)
        .into_model::<GetUserRentalHistoryAdminQueryResult>()
        .all(db)
        .await?;
    Ok(user_rental_history)
}

/// Deletes a rental history entry of the given ID from the database.
pub async fn delete_rental_history(id: i32, db: &DatabaseConnection) -> Result<(), DbErr> {
    RentalHistory::delete_by_id(id).exec(db).await?;
    Ok(())
}

/// Saves a rental history entry to the database. Handles both insertions and updates.
pub async fn save_extension_request(
    extension_request: ExtensionRequestActiveModel,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    extension_request.save(db).await?;
    Ok(())
}

/// Retrieves an extension request of the given ID from the database.
pub async fn get_extension_request(
    id: i32,
    db: &DatabaseConnection,
) -> Result<Option<ExtensionRequestModel>, DbErr> {
    let extension_request = ExtensionRequest::find_by_id(id).one(db).await?;
    Ok(extension_request)
}

/// Deletes an extension request of the given ID from the database.
pub async fn delete_extension_request(id: i32, db: &DatabaseConnection) -> Result<(), DbErr> {
    ExtensionRequest::delete_by_id(id).exec(db).await?;
    Ok(())
}

/// Saves a favourite to the database. Handles both insertions and updates.
pub async fn save_favourite(
    favourite: FavouriteActiveModel,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    favourite.save(db).await?;
    Ok(())
}

/// Deletes a favourite of the given user ID and game ID from the database.
pub async fn delete_favourite(
    user_id: i32,
    game_id: i32,
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    Favourite::delete_by_id((user_id, game_id)).exec(db).await?;
    Ok(())
}
