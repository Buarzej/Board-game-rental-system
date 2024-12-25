use futures::executor::block_on;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection, DbErr};
use tokio::runtime::Runtime;

const DB_NAME: &str = "database";
const DATABASE_URL: &str = "sqlite:./database.db?mode=rwc";

async fn initialize_database() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(DATABASE_URL).await?;
    Migrator::up(&db, None).await?;

    Ok(db)
}

fn main() {
    let rt = Runtime::new().expect("Failed to create Tokio runtime");
    
    // Connect to the database and run the migration.
    let db = rt.block_on(initialize_database()).expect("Failed to initialize database");
}
