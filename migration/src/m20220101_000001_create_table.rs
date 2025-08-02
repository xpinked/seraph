use sea_orm_migration::{
    prelude::{extension::postgres::Type, *},
    schema::*,
    sea_orm::{ActiveEnum, DbBackend, Schema},
};

use seraph_backend::code_nodes::{CodeLanguage, OutputType};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let schema = Schema::new(DbBackend::Postgres);

        // Create the `code_language` enum type
        manager
            .create_type(schema.create_enum_from_active_enum::<CodeLanguage>())
            .await?;

        manager
            .create_type(schema.create_enum_from_active_enum::<OutputType>())
            .await?;

        // Create the `code_nodes` table
        manager
            .create_table(
                Table::create()
                    .table(CodeNodes::Table) // Renamed from Posts to CodeNodes
                    .if_not_exists()
                    .col(pk_auto(CodeNodes::Id))
                    .col(string(CodeNodes::Name).not_null())
                    .col(string(CodeNodes::FunctionName).not_null())
                    .col(text(CodeNodes::Code).not_null())
                    .col(string(CodeNodes::OutputName).not_null())
                    .col(
                        ColumnDef::new(CodeNodes::OutputType)
                            .custom(OutputType::name())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CodeNodes::Language)
                            .custom(CodeLanguage::name())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the `code_nodes` table
        manager
            .drop_table(Table::drop().table(CodeNodes::Table).to_owned()) // Renamed from Posts to CodeNodes
            .await?;

        // Drop the `output_type` enum type
        manager
            .drop_type(Type::drop().name(OutputType::name()).to_owned())
            .await?;

        // Drop the `code_language` enum type
        manager
            .drop_type(Type::drop().name(CodeLanguage::name()).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum CodeNodes {
    // Renamed from Posts to CodeNodes
    Table,
    Id,
    Name,
    FunctionName,
    Code,
    OutputName,
    OutputType,
    Language,
}
