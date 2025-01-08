mod db_manager;

use crate::db_manager::initialize_database;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use migration::MigratorTrait;
use sea_orm::{DatabaseConnection, QueryTrait};

#[derive(Debug, Clone)]
struct AppState {
    db: DatabaseConnection,
}

struct RegisterFormData {
    name: String,
    surname: String,
    id: i32,
    email: String,
    password: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = initialize_database()
        .await
        .expect("Failed to initialize database");
    
    let state = AppState { db };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(web::scope("/api"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[post("/user/register")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}