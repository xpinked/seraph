use sea_orm_migration::{
    prelude::{extension::postgres::Type, *},
    sea_orm::{ActiveEnum, Schema},
};

use seraph_backend::code_result::Entity as CodeResultEntity;
use seraph_backend::enums::ResultStatus;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let builder = manager.get_database_backend();
        let schema = Schema::new(builder);
        // Create the `result_status` enum type
        manager.create_type(schema.create_enum_from_active_enum::<ResultStatus>()).await?;

        // Create the `code_results` table
        manager.create_table(schema.create_table_from_entity(CodeResultEntity)).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the `code_results` table
        manager.drop_table(Table::drop().table(CodeResultEntity).to_owned()).await?;
        // Drop the `result_status` enum type
        manager.drop_type(Type::drop().name(ResultStatus::name()).to_owned()).await?;
        Ok(())
    }
}
