use sea_orm_migration::{
    prelude::{extension::postgres::Type, *},
    schema::*,
    sea_orm::{ActiveEnum, DbBackend, Schema},
};

use super::m20220101_000001_create_table::CodeNodes;
use seraph_backend::enums::ResultStatus;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let schema = Schema::new(DbBackend::Postgres);
        // Create the `result_status` enum type
        manager.create_type(schema.create_enum_from_active_enum::<ResultStatus>()).await?;

        // Create the `code_results` table
        manager
            .create_table(
                Table::create()
                    .table(CodeResults::Table)
                    .if_not_exists()
                    .col(pk_auto(CodeResults::Id))
                    .col(integer(CodeResults::CodeNodeId).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_code_results_code_node_id")
                            .from(CodeResults::Table, CodeResults::CodeNodeId)
                            .to(CodeNodes::Table, CodeNodes::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(CodeResults::Status).custom(ResultStatus::name()).not_null())
                    .col(text_null(CodeResults::Output))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the `code_results` table
        manager.drop_table(Table::drop().table(CodeResults::Table).to_owned()).await?;
        // Drop the `result_status` enum type
        manager.drop_type(Type::drop().name(ResultStatus::name()).to_owned()).await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum CodeResults {
    Table,
    Id,
    CodeNodeId,
    Status,
    Output,
}
