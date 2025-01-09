mod auth;
mod db_manager;

use crate::auth::{generate_jwt, hash_password, verify_jwt, verify_password, Claims};
use crate::db_manager::DatabaseManager;
use actix_files::NamedFile;
use actix_web::dev::Payload;
use actix_web::http::header;
use actix_web::{get, post, web, App, FromRequest, HttpRequest, HttpResponse, HttpServer};
use dotenv::dotenv;
use entity::user::ActiveModel as UserActiveModel;
use futures::future::{ready, Ready};
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone)]
struct AppState {
    db: DatabaseManager,
}

struct Auth(Claims);

impl FromRequest for Auth {
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
                Ok(claims) => ready(Ok(Auth(claims))),
                Err(_) => ready(Err(actix_web::error::ErrorUnauthorized("Invalid token"))),
            },
            None => ready(Err(actix_web::error::ErrorUnauthorized(
                "No token provided",
            ))),
        }
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
            .service(index_register)
            .service(index_login)
            .service(secret)
            .service(web::scope("/api").service(register).service(login))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

// TODO: only for testing purposes
#[get("/login")]
async fn index_login(_req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = "./src/static/login.html".parse()?;
    Ok(NamedFile::open(path)?)
}

// TODO: only for testing purposes
#[get("/register")]
async fn index_register(_req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = "./src/static/register.html".parse()?;
    Ok(NamedFile::open(path)?)
}

#[get("/secret")]
async fn secret(Auth(user): Auth) -> HttpResponse {
    HttpResponse::Ok().body(format!("Hello, {}!", user.sub))
}

#[post("/user/login")]
async fn login(form: web::Form<LoginFormData>, data: web::Data<AppState>) -> HttpResponse {
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
async fn register(form: web::Form<RegisterFormData>, data: web::Data<AppState>) -> HttpResponse {
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
