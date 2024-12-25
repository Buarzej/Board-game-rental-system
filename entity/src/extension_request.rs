//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "extension_request")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub rental_id: i32,
    pub request_date: Date,
    pub extension_date: Date,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::rental::Entity",
        from = "Column::RentalId",
        to = "super::rental::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Rental,
}

impl Related<super::rental::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Rental.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
