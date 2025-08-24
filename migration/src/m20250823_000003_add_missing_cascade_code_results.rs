use sea_orm::entity::prelude::*;
use sea_orm::sea_query::ForeignKeyAction;
use sea_orm_migration::prelude::*;

use seraph_core::code_result::{Entity as CodeResultEntity, Relation as CodeResultRelation};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Alter the `code_results` table to add ON DELETE CASCADE to the foreign key

        let _key_name = CodeResultRelation::CodeNode.def().fk_name.unwrap();
        let _from = CodeResultRelation::CodeNode.def().from_col;
        let _to = CodeResultRelation::CodeNode.def().to_col;

        manager
            .drop_foreign_key(ForeignKey::drop().name(&_key_name).table(CodeResultEntity).to_owned())
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(_key_name)
                    .from(CodeResultEntity, _from)
                    .to(seraph_core::code_nodes::Entity, _to)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Revert the foreign key change by dropping and recreating it without ON DELETE CASCADE
        let _key_name = CodeResultRelation::CodeNode.def().fk_name.unwrap();
        let _from = CodeResultRelation::CodeNode.def().from_col;
        let _to = CodeResultRelation::CodeNode.def().to_col;

        manager
            .drop_foreign_key(ForeignKey::drop().name(&_key_name).table(CodeResultEntity).to_owned())
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(_key_name)
                    .from(CodeResultEntity, _from)
                    .to(seraph_core::code_nodes::Entity, _to)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
