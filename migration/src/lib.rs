pub use sea_orm_migration::prelude::*;
mod m20241225_172304_create_board_games_table;
mod m20241225_180742_create_users_table;
mod m20241225_182718_create_rentals_table;
mod m20241225_190739_create_rental_history_table;
mod m20241225_191215_create_extension_requests_table;
mod m20241226_145812_create_favourites_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241225_172304_create_board_games_table::Migration),
            Box::new(m20241225_180742_create_users_table::Migration),
            Box::new(m20241225_182718_create_rentals_table::Migration),
            Box::new(m20241225_190739_create_rental_history_table::Migration),
            Box::new(m20241225_191215_create_extension_requests_table::Migration),
            Box::new(m20241226_145812_create_favourites_table::Migration),
        ]
    }
}