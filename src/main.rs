mod auth;
mod db_manager;

use crate::auth::{
    generate_jwt, hash_password, send_confirmation_email, verify_jwt, verify_password, Claims,
};
use crate::db_manager::DatabaseManager;
use actix_files::{Files, NamedFile};
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::text::Text;
use actix_multipart::form::MultipartForm;
use actix_web::dev::Payload;
use actix_web::http::header;
use actix_web::web::{Data, Form};
use actix_web::{get, post, web, App, FromRequest, HttpRequest, HttpResponse, HttpServer};
use chrono::NaiveDate as Date;
use dotenv::dotenv;
use entity::board_game::ActiveModel as BoardGameActiveModel;
use entity::favourite::ActiveModel as FavouriteActiveModel;
use entity::rental::ActiveModel as RentalActiveModel;
use entity::user::ActiveModel as UserActiveModel;
use futures::future::{ready, Ready};
use sea_orm::ActiveValue::Set;
use sea_orm::NotSet;
use serde::Deserialize;
use std::path::PathBuf;
use uuid::Uuid;

const HAS_TOKEN: bool = false;
const HAS_ADMIN_TOKEN: bool = true;
const REQUIRED_ENV_VARS: [&str; 7] = [
    "JWT_SECRET",
    "MAILER_HOST",
    "MAILER_PORT",
    "MAILER_USERNAME",
    "MAILER_EMAIL",
    "MAILER_PASSWORD",
    "FRONTEND_URL",
];

#[derive(Debug, Clone)]
struct AppState {
    db: DatabaseManager,
}

// Struct for authenticating clients' requests.
struct Auth<const IS_ADMIN: bool>(Claims);

impl<const IS_ADMIN: bool> FromRequest for Auth<IS_ADMIN> {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let token = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|str| str.split(" ").nth(1)); // Token is the second word.

        match token {
            Some(token) => match verify_jwt(token) {
                Ok(claims) => {
                    if IS_ADMIN && !claims.is_admin {
                        return ready(Err(actix_web::error::ErrorForbidden(
                            "Insufficient privileges",
                        )));
                    }
                    ready(Ok(Self(claims)))
                }
                Err(e) => ready(Err(actix_web::error::ErrorUnauthorized(format!(
                    "Invalid token: {}",
                    e
                )))),
            },
            None => ready(Err(actix_web::error::ErrorUnauthorized(
                "No token provided",
            ))),
        }
    }
}

fn is_self_request(user: &Claims, id: i32) -> Result<(), HttpResponse> {
    if !user.is_admin && user.sub != id {
        Err(HttpResponse::Forbidden().body("Insufficient privileges"))
    } else {
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct LoginFormData {
    id: i32,
    password: String,
}

#[derive(Debug, Deserialize)]
struct RegisterFormData {
    name: String,
    surname: String,
    id: i32,
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct ChangePasswordFormData {
    password: String,
}

#[derive(Debug, Deserialize)]
struct UpdateUserFormData {
    name: String,
    surname: String,
    email: String,
    password: String,
    penalty_points: u8,
    is_admin: bool,
}

#[derive(Debug, MultipartForm)]
struct BoardGameFormData {
    title: Text<String>,
    weight: Text<u16>,
    image: TempFile,
    min_players: Text<u8>,
    max_players: Text<u8>,
    min_playtime: Text<u16>,
    max_playtime: Text<u16>,
    additional_info: Text<String>,
}

#[derive(Debug, Deserialize)]
struct RentalFormData {
    game_id: i32,
    rental_date: String,
    return_date: String,
}

#[derive(Debug, Deserialize)]
struct ExtensionRequestFormData {
    extension_date: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load all the necessary resources.
    dotenv().ok();
    for var in REQUIRED_ENV_VARS.iter() {
        if std::env::var(var).is_err() {
            panic!("{} is not set", var);
        }
    }
    let db = DatabaseManager::new(&std::env::var("DATABASE_URL").expect("DATABASE_URL is not set"))
        .await
        .expect("Failed to initialize database");

    let state = AppState { db };

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(state.clone()))
            .service(Files::new("/static", "./static/img"))
            .service(index_test)
            .service(index_login)
            .service(index_register)
            .service(index_board_game)
            .service(
                web::scope("/api")
                    .service(login)
                    .service(register)
                    .service(confirm_user)
                    .service(get_user)
                    .service(get_users)
                    .service(is_penalized)
                    .service(change_password)
                    .service(update_user)
                    .service(delete_user)
                    .service(save_board_game)
                    .service(get_board_game)
                    .service(get_board_games)
                    .service(get_board_games_admin)
                    .service(delete_board_game)
                    .service(save_rental)
                    .service(get_rentals)
                    .service(get_my_rentals)
                    .service(get_user_rentals)
                    .service(archive_rental)
                    .service(get_rental_history)
                    .service(get_my_rental_history)
                    .service(get_user_rental_history)
                    .service(delete_rental_history)
                    .service(save_extension_request)
                    .service(accept_extension_request)
                    .service(delete_extension_request)
                    .service(save_favourite)
                    .service(delete_favourite),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

// For now, only for testing purposes.
#[get("/test")]
async fn index_test(_req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = "./static/test.html".parse()?;
    Ok(NamedFile::open(path)?)
}

// For now, only for testing purposes.
#[get("/login")]
async fn index_login(_req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = "./static/login.html".parse()?;
    Ok(NamedFile::open(path)?)
}

// For now, only for testing purposes.
#[get("/register")]
async fn index_register(_req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = "./static/register.html".parse()?;
    Ok(NamedFile::open(path)?)
}

// For now, only for testing purposes.
#[get("/board_game")]
async fn index_board_game(_req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = "./static/add_board_game.html".parse()?;
    Ok(NamedFile::open(path)?)
}

#[post("/user/login")]
async fn login(form: Form<LoginFormData>, data: Data<AppState>) -> HttpResponse {
    match data.db.get_user(form.id).await {
        Ok(Some(user)) => {
            if user.confirmation_token.is_some() {
                HttpResponse::Unauthorized().body("User not confirmed")
            } else {
                match verify_password(form.password.clone(), user.password_hash) {
                    Ok(true) => match generate_jwt(user.id, user.is_admin) {
                        Ok(token) => HttpResponse::Ok().body(token),
                        Err(_) => {
                            HttpResponse::InternalServerError().body("Failed to generate JWT")
                        }
                    },
                    Ok(false) => HttpResponse::Unauthorized().body("Invalid password"),
                    Err(_) => HttpResponse::InternalServerError().body("Failed to verify password"),
                }
            }
        }
        Ok(None) => HttpResponse::Unauthorized().body("User not found"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to get user data from database"),
    }
}

#[post("/user/register")]
async fn register(form: Form<RegisterFormData>, data: Data<AppState>) -> HttpResponse {
    let password_hash = match hash_password(form.password.clone()) {
        Ok(hash) => hash,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to hash password"),
    };

    let uuid = Uuid::new_v4();
    let user = UserActiveModel {
        id: Set(form.id),
        name: Set(form.name.clone()),
        surname: Set(form.surname.clone()),
        email: Set(form.email.clone()),
        password_hash: Set(password_hash),
        // TODO: fix emails
        // confirmation_token: Set(Some(uuid)),
        confirmation_token: Set(None),
        ..Default::default()
    };

    match data.db.insert_user(user).await {
        // TODO: fix emails
        // Ok(_) => match send_confirmation_email(form.id, form.email.clone(), uuid) {
        //     Ok(_) => HttpResponse::Ok().body("User registered"),
        //     Err(e) => {
        //         HttpResponse::InternalServerError().body(format!("Failed to send email: {}", e))
        //     }
        // },
        Ok(_) => HttpResponse::Ok().body("User registered"),
        Err(_) => {
            HttpResponse::InternalServerError().body("Failed to save user data into database")
        }
    }
}

#[get("/user/confirm/{id}/{token}")]
async fn confirm_user(path: web::Path<(i32, Uuid)>, data: Data<AppState>) -> HttpResponse {
    let (id, token) = path.into_inner();
    let user = match data.db.get_user(id).await {
        Ok(Some(user)) => user,
        Ok(None) => return HttpResponse::NotFound().body("User not found"),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Failed to get user data from database")
        }
    };

    if user.confirmation_token != Some(token) {
        return HttpResponse::Unauthorized().body("Invalid token");
    }

    let user = UserActiveModel {
        id: Set(id),
        confirmation_token: Set(None),
        ..Default::default()
    };

    match data.db.update_user(user).await {
        Ok(_) => HttpResponse::Ok().body("User confirmed"),
        Err(_) => {
            HttpResponse::InternalServerError().body("Failed to save user data into database")
        }
    }
}

#[get("/user/get/{id}")]
async fn get_user(
    id: web::Path<i32>,
    Auth(user): Auth<HAS_TOKEN>,
    data: Data<AppState>,
) -> HttpResponse {
    // Non-admin user can only check info about themselves.
    let id = id.into_inner();
    if let Err(response) = is_self_request(&user, id) {
        return response;
    }

    match data.db.get_user(id).await {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => HttpResponse::NotFound().body("User not found"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to get user data from database"),
    }
}

#[get("/user/get_all")]
async fn get_users(Auth(_user): Auth<HAS_ADMIN_TOKEN>, data: Data<AppState>) -> HttpResponse {
    match data.db.get_users().await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(_) => {
            HttpResponse::InternalServerError().body("Failed to get users data from database")
        }
    }
}

#[get("/user/is_penalized/{id}")]
async fn is_penalized(
    id: web::Path<i32>,
    Auth(user): Auth<HAS_TOKEN>,
    data: Data<AppState>,
) -> HttpResponse {
    // Non-admin user can only check info about themselves.
    let id = id.into_inner();
    if let Err(response) = is_self_request(&user, id) {
        return response;
    }

    match data.db.is_user_penalized(id).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(_) => HttpResponse::InternalServerError().body("Failed to get user data from database"),
    }
}

#[post("/user/change_password/{id}")]
async fn change_password(
    id: web::Path<i32>,
    form: Form<ChangePasswordFormData>,
    Auth(user): Auth<HAS_TOKEN>,
    data: Data<AppState>,
) -> HttpResponse {
    // Non-admin user can only change their own password.
    let id = id.into_inner();
    if let Err(response) = is_self_request(&user, id) {
        return response;
    }

    let password_hash = match hash_password(form.password.clone()) {
        Ok(hash) => hash,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to hash password"),
    };

    let user = UserActiveModel {
        id: Set(id),
        password_hash: Set(password_hash),
        ..Default::default()
    };

    match data.db.update_user(user).await {
        Ok(_) => HttpResponse::Ok().into(),
        Err(_) => {
            HttpResponse::InternalServerError().body("Failed to save user data into database")
        }
    }
}

#[post("/user/update/{id}")]
async fn update_user(
    id: web::Path<i32>,
    form: Form<UpdateUserFormData>,
    Auth(_user): Auth<HAS_ADMIN_TOKEN>,
    data: Data<AppState>,
) -> HttpResponse {
    let id = id.into_inner();

    let password_hash = if !form.password.is_empty() {
        NotSet
    } else if let Ok(hash) = hash_password(form.password.clone()) {
        Set(hash)
    } else {
        return HttpResponse::InternalServerError().body("Failed to hash password");
    };

    let user = UserActiveModel {
        id: Set(id),
        name: Set(form.name.clone()),
        surname: Set(form.surname.clone()),
        email: Set(form.email.clone()),
        password_hash,
        penalty_points: Set(form.penalty_points),
        is_admin: Set(form.is_admin),
        ..Default::default()
    };

    match data.db.update_user(user).await {
        Ok(_) => HttpResponse::Ok().into(),
        Err(_) => {
            HttpResponse::InternalServerError().body("Failed to save user data into database")
        }
    }
}

#[get("/user/delete/{id}")]
async fn delete_user(
    id: web::Path<i32>,
    Auth(_user): Auth<HAS_ADMIN_TOKEN>,
    data: Data<AppState>,
) -> HttpResponse {
    let id = id.into_inner();
    match data.db.delete_user(id).await {
        Ok(_) => HttpResponse::Ok().into(),
        Err(_) => {
            HttpResponse::InternalServerError().body("Failed to delete user from database")
        }
    }
}

/// id = 0 ==> insert new board game
#[post("/board_game/save/{id}")]
async fn save_board_game(
    id: web::Path<i32>,
    MultipartForm(form): MultipartForm<BoardGameFormData>,
    // TODO: uncomment this
    // Auth(_user): Auth<HAS_ADMIN_TOKEN>,
    data: Data<AppState>,
) -> HttpResponse {
    let file_name = match form.image.file_name {
        Some(name) => name,
        None => return HttpResponse::BadRequest().body("File name is missing"),
    };

    // Save the image on the server.
    let path = format!("./static/img/{}", file_name);
    if form.image.file.persist(path).is_err() {
        return HttpResponse::InternalServerError().body("Failed to save file");
    }

    let id = id.into_inner();
    let additional_info = form.additional_info.into_inner();
    let additional_info = if additional_info.is_empty() {
        None
    } else {
        Some(additional_info)
    };
    let board_game = BoardGameActiveModel {
        id: if id == 0 { NotSet } else { Set(id) },
        title: Set(form.title.into_inner()),
        weight: Set(form.weight.into_inner()),
        photo_filename: Set(file_name),
        min_players: Set(form.min_players.into_inner()),
        max_players: Set(form.max_players.into_inner()),
        min_playtime: Set(form.min_playtime.into_inner()),
        max_playtime: Set(form.max_playtime.into_inner()),
        additional_info: Set(additional_info),
    };

    match data.db.save_board_game(board_game).await {
        Ok(_) => HttpResponse::Ok().into(),
        Err(_) => {
            HttpResponse::InternalServerError().body("Failed to save board game into database")
        }
    }
}

#[get("/board_game/get/{id}")]
async fn get_board_game(
    id: web::Path<i32>,
    Auth(_user): Auth<HAS_TOKEN>,
    data: Data<AppState>,
) -> HttpResponse {
    let id = id.into_inner();
    match data.db.get_board_game(id).await {
        Ok(Some(board_game)) => HttpResponse::Ok().json(board_game),
        Ok(None) => HttpResponse::NotFound().body("Board game not found"),
        Err(_) => {
            HttpResponse::InternalServerError().body("Failed to get board game data from database")
        }
    }
}

#[get("/board_game/get_all")]
async fn get_board_games(Auth(user): Auth<HAS_TOKEN>, data: Data<AppState>) -> HttpResponse {
    match data.db.get_board_games(user.sub).await {
        Ok(board_games) => HttpResponse::Ok().json(board_games),
        Err(_) => {
            HttpResponse::InternalServerError().body("Failed to get board games data from database")
        }
    }
}

#[get("/board_game/get_all_admin")]
async fn get_board_games_admin(
    Auth(_user): Auth<HAS_ADMIN_TOKEN>,
    data: Data<AppState>,
) -> HttpResponse {
    match data.db.get_board_games_admin().await {
        Ok(board_games) => HttpResponse::Ok().json(board_games),
        Err(_) => {
            HttpResponse::InternalServerError().body("Failed to get board games data from database")
        }
    }
}

#[get("/board_game/delete/{id}")]
async fn delete_board_game(
    id: web::Path<i32>,
    Auth(_user): Auth<HAS_ADMIN_TOKEN>,
    data: Data<AppState>,
) -> HttpResponse {
    let id = id.into_inner();
    match data.db.delete_board_game(id).await {
        Ok(_) => HttpResponse::Ok().into(),
        Err(_) => {
            HttpResponse::InternalServerError().body("Failed to delete board game from database")
        }
    }
}

/// id = 0 ==> insert new rental
#[post("/rental/save/{id}")]
async fn save_rental(
    id: web::Path<i32>,
    form: Form<RentalFormData>,
    Auth(user): Auth<HAS_TOKEN>,
    data: Data<AppState>,
) -> HttpResponse {
    
    // TODO: users can only add or update their own rentals
    
    let id = id.into_inner();
    let rental_date = match Date::parse_from_str(form.rental_date.as_str(), "%Y-%m-%d") {
        Ok(date) => date,
        Err(_) => return HttpResponse::BadRequest().body("Invalid date format"),
    };
    let return_date = match Date::parse_from_str(form.return_date.as_str(), "%Y-%m-%d") {
        Ok(date) => date,
        Err(_) => return HttpResponse::BadRequest().body("Invalid date format"),
    };

    let rental = RentalActiveModel {
        id: if id == 0 { NotSet } else { Set(id) },
        game_id: Set(form.game_id),
        user_id: Set(user.sub),
        rental_date: Set(rental_date),
        return_date: Set(return_date),
        extension_date: Set(None),
        ..Default::default()
    };

    match data.db.save_rental(rental).await {
        Ok(_) => HttpResponse::Ok().into(),
        Err(_) => HttpResponse::InternalServerError().body("Failed to save rental into database"),
    }
}

#[get("/rental/get_all")]
async fn get_rentals(Auth(_user): Auth<HAS_ADMIN_TOKEN>, data: Data<AppState>) -> HttpResponse {
    match data.db.get_rentals().await {
        Ok(rentals) => HttpResponse::Ok().json(rentals),
        Err(_) => HttpResponse::InternalServerError().body("Failed to get rentals from database"),
    }
}

#[get("/rental/get")]
async fn get_my_rentals(Auth(user): Auth<HAS_TOKEN>, data: Data<AppState>) -> HttpResponse {
    match data.db.get_user_rentals(user.sub).await {
        Ok(rentals) => HttpResponse::Ok().json(rentals),
        Err(_) => HttpResponse::InternalServerError().body("Failed to get rentals from database"),
    }
}

#[get("/rental/get/{id}")]
async fn get_user_rentals(
    id: web::Path<i32>,
    Auth(_user): Auth<HAS_ADMIN_TOKEN>,
    data: Data<AppState>,
) -> HttpResponse {
    let id = id.into_inner();
    match data.db.get_user_rentals_admin(id).await {
        Ok(rentals) => HttpResponse::Ok().json(rentals),
        Err(_) => HttpResponse::InternalServerError().body("Failed to get rentals from database"),
    }
}

#[get("/rental/archive/{id}")]
async fn archive_rental(
    id: web::Path<i32>,
    Auth(user): Auth<HAS_TOKEN>,
    data: Data<AppState>,
) -> HttpResponse {
    // Non-admin user can only archive their own rentals that have not been picked up.
    let id = id.into_inner();
    if !user.is_admin {
        let rental = match data.db.get_rental(id).await {
            Ok(Some(rental)) => rental,
            Ok(None) => return HttpResponse::NotFound().body("Rental not found"),
            Err(_) => {
                return HttpResponse::InternalServerError()
                    .body("Failed to get rental data from database")
            }
        };
        if rental.user_id != user.sub || rental.picked_up {
            return HttpResponse::Forbidden().body("Insufficient privileges");
        }
    }

    match data.db.archive_rental(id).await {
        Ok(_) => HttpResponse::Ok().into(),
        Err(_) => HttpResponse::InternalServerError().body("Failed to delete rental from database"),
    }
}

#[get("/history/get_all")]
async fn get_rental_history(
    Auth(_user): Auth<HAS_ADMIN_TOKEN>,
    data: Data<AppState>,
) -> HttpResponse {
    match data.db.get_rental_history().await {
        Ok(rental_history) => HttpResponse::Ok().json(rental_history),
        Err(_) => HttpResponse::InternalServerError()
            .body("Failed to get rental history data from database"),
    }
}

#[get("/history/get")]
async fn get_my_rental_history(Auth(user): Auth<HAS_TOKEN>, data: Data<AppState>) -> HttpResponse {
    match data.db.get_user_rental_history(user.sub).await {
        Ok(rental_history) => HttpResponse::Ok().json(rental_history),
        Err(_) => HttpResponse::InternalServerError()
            .body("Failed to get rental history data from database"),
    }
}

#[get("/history/get/{id}")]
async fn get_user_rental_history(
    id: web::Path<i32>,
    Auth(_user): Auth<HAS_ADMIN_TOKEN>,
    data: Data<AppState>,
) -> HttpResponse {
    let id = id.into_inner();
    match data.db.get_user_rental_history_admin(id).await {
        Ok(rental_history) => HttpResponse::Ok().json(rental_history),
        Err(_) => HttpResponse::InternalServerError()
            .body("Failed to get rental history data from database"),
    }
}

#[get("/history/delete/{id}")]
async fn delete_rental_history(
    id: web::Path<i32>,
    Auth(_user): Auth<HAS_ADMIN_TOKEN>,
    data: Data<AppState>,
) -> HttpResponse {
    let id = id.into_inner();
    match data.db.delete_rental_history(id).await {
        Ok(_) => HttpResponse::Ok().into(),
        Err(_) => HttpResponse::InternalServerError()
            .body("Failed to delete rental history data from database"),
    }
}

#[post("/extension/save/{id}")]
async fn save_extension_request(
    id: web::Path<i32>,
    form: Form<ExtensionRequestFormData>,
    Auth(user): Auth<HAS_TOKEN>,
    data: Data<AppState>,
) -> HttpResponse {
    let rental_id = id.into_inner();
    let rental = match data.db.get_rental(rental_id).await {
        Ok(Some(rental)) => rental,
        Ok(None) => return HttpResponse::NotFound().body("Related rental not found"),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Failed to get rental data from database")
        }
    };

    // Non-admin user can only modify their own rentals.
    if !user.is_admin && rental.user_id != user.sub {
        return HttpResponse::Forbidden().body("Insufficient privileges");
    }

    let extension_date = match Date::parse_from_str(form.extension_date.as_str(), "%Y-%m-%d") {
        Ok(date) => date,
        Err(_) => return HttpResponse::BadRequest().body("Invalid date format"),
    };
    let rental = RentalActiveModel {
        id: Set(rental_id),
        extension_date: Set(Some(extension_date)),
        ..Default::default()
    };

    match data.db.save_rental(rental).await {
        Ok(_) => HttpResponse::Ok().into(),
        Err(_) => HttpResponse::InternalServerError()
            .body("Failed to save extension request into database"),
    }
}

#[get("/extension/accept/{id}")]
async fn accept_extension_request(
    id: web::Path<i32>,
    Auth(_user): Auth<HAS_ADMIN_TOKEN>,
    data: Data<AppState>,
) -> HttpResponse {
    let rental_id = id.into_inner();
    let rental = match data.db.get_rental(rental_id).await {
        Ok(Some(rental)) => rental,
        Ok(None) => return HttpResponse::NotFound().body("Related rental not found"),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Failed to get rental data from database")
        }
    };

    let new_date = match rental.extension_date {
        Some(date) => date,
        None => return HttpResponse::BadRequest().body("Extension request not found"),
    };

    let rental = RentalActiveModel {
        id: Set(rental_id),
        return_date: Set(new_date),
        extension_date: Set(None),
        ..Default::default()
    };

    match data.db.save_rental(rental).await {
        Ok(_) => HttpResponse::Ok().into(),
        Err(_) => HttpResponse::InternalServerError()
            .body("Failed to save extension request into database"),
    }
}

#[get("/extension/delete/{id}")]
async fn delete_extension_request(
    id: web::Path<i32>,
    Auth(user): Auth<HAS_TOKEN>,
    data: Data<AppState>,
) -> HttpResponse {
    let rental_id = id.into_inner();
    let rental = match data.db.get_rental(rental_id).await {
        Ok(Some(rental)) => rental,
        Ok(None) => return HttpResponse::NotFound().body("Related rental not found"),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Failed to get rental data from database")
        }
    };

    // Non-admin user can only modify their own rentals.
    if !user.is_admin && rental.user_id != user.sub {
        return HttpResponse::Forbidden().body("Insufficient privileges");
    }

    let rental = RentalActiveModel {
        id: Set(rental_id),
        extension_date: Set(None),
        ..Default::default()
    };

    match data.db.save_rental(rental).await {
        Ok(_) => HttpResponse::Ok().into(),
        Err(_) => HttpResponse::InternalServerError()
            .body("Failed to save extension request into database"),
    }
}

#[get("/favourite/save/{id}")]
async fn save_favourite(
    id: web::Path<i32>,
    Auth(user): Auth<HAS_TOKEN>,
    data: Data<AppState>,
) -> HttpResponse {
    let game_id = id.into_inner();
    let favourite = FavouriteActiveModel {
        user_id: Set(user.sub),
        game_id: Set(game_id),
    };

    match data.db.save_favourite(favourite).await {
        Ok(_) => HttpResponse::Ok().into(),
        Err(_) => {
            HttpResponse::InternalServerError().body("Failed to save favourite into database")
        }
    }
}

#[get("/favourite/delete/{id}")]
async fn delete_favourite(
    id: web::Path<i32>,
    Auth(user): Auth<HAS_TOKEN>,
    data: Data<AppState>,
) -> HttpResponse {
    let game_id = id.into_inner();
    match data.db.delete_favourite(user.sub, game_id).await {
        Ok(_) => HttpResponse::Ok().into(),
        Err(_) => {
            HttpResponse::InternalServerError().body("Failed to delete favourite from database")
        }
    }
}
