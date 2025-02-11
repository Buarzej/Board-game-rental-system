//! `SeaORM` Entity, partially @generated by sea-orm-codegen 1.1.3

use sea_orm::entity::prelude::*;
use sea_orm::prelude::async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "board_game")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,
    pub title: String,
    pub weight: u16,
    pub photo_filename: String,
    pub min_players: u8,
    pub max_players: u8,
    pub min_playtime: u16,
    pub max_playtime: u16,
    #[sea_orm(column_type = "Text", nullable)]
    pub additional_info: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::favourite::Entity")]
    Favourite,
    #[sea_orm(has_one = "super::rental::Entity")]
    Rental,
    #[sea_orm(has_many = "super::rental_history::Entity")]
    RentalHistory,
}

impl Related<super::favourite::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Favourite.def()
    }
}

impl Related<super::rental::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Rental.def()
    }
}

impl Related<super::rental_history::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RentalHistory.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        super::favourite::Relation::User.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::favourite::Relation::BoardGame.def().rev())
    }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, _insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if self.min_players.as_ref() > self.max_players.as_ref() {
            return Err(DbErr::Custom("min_players cannot be greater than max_players".into()));
        }

        if self.min_playtime.as_ref() > self.max_playtime.as_ref() {
            return Err(DbErr::Custom("min_playtime cannot be greater than max_playtime".into()));
        }
        
        if self.weight.as_ref() == &0 {
            return Err(DbErr::Custom("weight cannot be equal to 0".into()));
        }

        Ok(self)
    }
}
