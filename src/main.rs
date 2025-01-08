mod auth_manager;
mod db_manager;

use crate::auth_manager::AuthManager;
use crate::db_manager::DatabaseManager;
use actix_files::NamedFile;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use dotenv::dotenv;
use entity::user::ActiveModel as UserActiveModel;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone)]
struct AppState {
    db: DatabaseManager,
    auth: AuthManager,
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
    dotenv().ok(); // Load environment variables
    let db = DatabaseManager::new(&std::env::var("DATABASE_URL").expect("DATABASE_URL is not set"))
        .await
        .expect("Failed to initialize database");
    let auth = AuthManager::new().expect("Failed to initialize auth manager");

    let state = AppState { db, auth };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(index_register)
            .service(index_login)
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

#[post("/user/login")]
async fn login(form: web::Form<LoginFormData>, data: web::Data<AppState>) -> HttpResponse {
    match data.db.get_user(form.id).await {
        Ok(Some(user)) => {
            match data
                .auth
                .verify_password(form.password.clone(), user.password_hash)
            {
                Ok(true) => match data.auth.generate_jwt(user.id, user.is_admin) {
                    Ok(token) => HttpResponse::Ok().body(token),
                    Err(_) => HttpResponse::InternalServerError().body("Failed to generate JWT"),
                },
                Ok(false) => HttpResponse::Unauthorized().body("Invalid password"),
                Err(_) => HttpResponse::InternalServerError().body("Failed to verify password"),
            }
        }
        Ok(None) => HttpResponse::Unauthorized().body("User not found"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to get user data from database"),
    }
}

#[post("/user/register")]
async fn register(form: web::Form<RegisterFormData>, data: web::Data<AppState>) -> HttpResponse {
    let password_hash = match data.auth.hash_password(form.password.clone()) {
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
