mod db_manager;

use crate::db_manager::{initialize_database};
use futures::executor::block_on;
use migration::MigratorTrait;

#[tokio::main]
async fn main() {
    // Connect to the database and run the migration.
    let db = block_on(initialize_database()).expect("Failed to initialize database");
}
