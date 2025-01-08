mod db_manager;

use crate::db_manager::DatabaseManager;
use actix_files::NamedFile;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use entity::user::ActiveModel as UserActiveModel;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const DATABASE_URL: &str = "sqlite:./database.db?mode=rwc";

#[derive(Debug, Clone)]
struct AppState {
    db: DatabaseManager,
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
    let db = DatabaseManager::new(DATABASE_URL)
        .await
        .expect("Failed to initialize database");

    let state = AppState { db };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(index_register)
            .service(index_login)
            .service(
                web::scope("/api")
                    .service(register)
                    .service(count)
                    .service(login),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[get("/login")]
async fn index_login(_req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = "./src/static/login.html".parse()?;
    Ok(NamedFile::open(path)?)
}

#[get("/register")]
async fn index_register(_req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = "./src/static/register.html".parse()?;
    Ok(NamedFile::open(path)?)
}

#[get("/user/count")]
async fn count(data: web::Data<AppState>) -> impl Responder {
    let users = data.db.get_users().await.expect("Failed to count users");
    HttpResponse::Ok().body(format!("User count: {}", users.len()))
}

#[post("/user/login")]
async fn login(form: web::Form<LoginFormData>, data: web::Data<AppState>) -> HttpResponse {
    match data.db.get_user(form.id).await {
        Ok(Some(user)) => {
            let parsed_hash = match PasswordHash::new(&user.password_hash) {
                Ok(parsed_hash) => parsed_hash,
                Err(_) => {
                    return HttpResponse::InternalServerError()
                        .body("Failed to parse password hash")
                }
            };
            match Argon2::default().verify_password(form.password.as_bytes(), &parsed_hash) {
                Ok(_) => HttpResponse::Ok().into(), // TODO: add JWT token
                Err(_) => HttpResponse::Unauthorized().body("Invalid password"),
            }
        }
        Ok(None) => HttpResponse::Unauthorized().body("User not found"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to get user data from database"),
    }
}

#[post("/user/register")]
async fn register(form: web::Form<RegisterFormData>, data: web::Data<AppState>) -> HttpResponse {
    // Hash user's password using Argon2 algorithm.
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = match argon2.hash_password(form.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(_) => {
            return HttpResponse::InternalServerError().body("Failed to hash password with Argon2")
        }
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
