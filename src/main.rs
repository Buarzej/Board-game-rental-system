use futures::executor::block_on;
use sea_orm::{Database, DbErr};

const DATABASE_URL: &str = "sqlite:./database.db?mode=rwc";
const DB_NAME: &str = "database";

async fn run() -> Result<(), DbErr> {
    let db = Database::connect(DATABASE_URL).await?;

    Ok(())
}

fn main() {
    if let Err(err) = block_on(run()) {
        panic!("{}", err);
    } else {
        println!("Connected to database: {}", DB_NAME);
    }
}