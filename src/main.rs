use futures::executor::block_on;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DbErr};

const DB_NAME: &str = "database";
const DATABASE_URL: &str = "sqlite:./database.db?mode=rwc";

async fn initialize_database() -> Result<(), DbErr> {
    let db = Database::connect(DATABASE_URL).await?;
    Migrator::up(&db, None).await?;

    Ok(())
}

fn main() {
    // Connect to the database and run the migration.
    if let Err(err) = block_on(initialize_database()) {
        panic!("{}", err);
    }

    println!("Connection and migration successful");
}
