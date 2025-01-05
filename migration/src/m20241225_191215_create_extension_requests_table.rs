use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum ExtensionRequest {
    Table,
    Id,
    RentalId,
    RequestDate,
    ExtensionDate,
}

#[derive(DeriveIden)]
pub enum Rental {
    Table,
    Id,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ExtensionRequest::Table)
                    .if_not_exists()
                    .col(pk_auto(ExtensionRequest::Id))
                    .col(integer_uniq(ExtensionRequest::RentalId))
                    .col(date(ExtensionRequest::RequestDate))
                    .col(date(ExtensionRequest::ExtensionDate))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_extension_request_rental")
                            .from(ExtensionRequest::Table, ExtensionRequest::RentalId)
                            .to(Rental::Table, Rental::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ExtensionRequest::Table).to_owned())
            .await
    }
}
