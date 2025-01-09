mod auth;
mod db_manager;

use crate::auth::{generate_jwt, hash_password, verify_jwt, verify_password, Claims};
use crate::db_manager::DatabaseManager;
use actix_files::{Files, NamedFile};
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::text::Text;
use actix_multipart::form::MultipartForm;
use actix_web::dev::Payload;
use actix_web::http::header;
use actix_web::web::{Data, Form};
use actix_web::{get, post, web, App, FromRequest, HttpRequest, HttpResponse, HttpServer};
use dotenv::dotenv;
use entity::board_game::ActiveModel as BoardGameActiveModel;
use entity::user::ActiveModel as UserActiveModel;
use futures::future::{ready, Ready};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveValue, NotSet};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const HAS_TOKEN: bool = false;
const HAS_ADMIN_TOKEN: bool = true;

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
struct RegisterFormData {
    name: String,
    surname: String,
    id: i32,
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct LoginFormData {
    id: i32,
    password: String,
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load all the necessary resources.
    dotenv().ok();
    if std::env::var("JWT_SECRET").is_err() {
        panic!("JWT_SECRET is not set");
    }
    let db = DatabaseManager::new(&std::env::var("DATABASE_URL").expect("DATABASE_URL is not set"))
        .await
        .expect("Failed to initialize database");

    let state = AppState { db };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(Files::new("/static", "./static/img"))
            .service(index_login)
            .service(index_register)
            .service(index_board_game)
            .service(
                web::scope("/api")
                    .service(login)
                    .service(register)
                    .service(get_user)
                    .service(get_users)
                    .service(is_penalized)
                    .service(change_password)
                    .service(update_user)
                    .service(save_board_game),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
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
        Ok(Some(user)) => match verify_password(form.password.clone(), user.password_hash) {
            Ok(true) => match generate_jwt(user.id, user.is_admin) {
                Ok(token) => HttpResponse::Ok().body(token),
                Err(_) => HttpResponse::InternalServerError().body("Failed to generate JWT"),
            },
            Ok(false) => HttpResponse::Unauthorized().body("Invalid password"),
            Err(_) => HttpResponse::InternalServerError().body("Failed to verify password"),
        },
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

    let user = UserActiveModel {
        id: Set(form.id),
        name: Set(form.name.clone()),
        surname: Set(form.surname.clone()),
        email: Set(form.email.clone()),
        password_hash: Set(password_hash),
        ..Default::default()
    };

    match data.db.insert_user(user).await {
        Ok(_) => HttpResponse::Ok().into(),
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
    };

    match data.db.update_user(user).await {
        Ok(_) => HttpResponse::Ok().into(),
        Err(_) => {
            HttpResponse::InternalServerError().body("Failed to save user data into database")
        }
    }
}

#[post("/board_game/save")]
async fn save_board_game(
    MultipartForm(form): MultipartForm<BoardGameFormData>,
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

    let additional_info = form.additional_info.into_inner();
    let additional_info = if additional_info.is_empty() {
        None
    } else {
        Some(additional_info)
    };
    let board_game = BoardGameActiveModel {
        title: Set(form.title.into_inner()),
        weight: Set(form.weight.into_inner()),
        photo_filename: Set(file_name),
        min_players: Set(form.min_players.into_inner()),
        max_players: Set(form.max_players.into_inner()),
        min_playtime: Set(form.min_playtime.into_inner()),
        max_playtime: Set(form.max_playtime.into_inner()),
        additional_info: Set(additional_info),
        ..Default::default()
    };

    match data.db.save_board_game(board_game).await {
        Ok(_) => HttpResponse::Ok().into(),
        Err(_) => {
            HttpResponse::InternalServerError().body("Failed to save board game into database")
        }
    }
}
