use crate::enums::ResultStatus;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "code_results")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,
    pub task_id: uuid::Uuid,
    pub code_node_id: i32,
    pub status: ResultStatus,
    pub output: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::code_nodes::Entity",
        from = "Column::CodeNodeId",
        to = "super::code_nodes::Column::Id"
    )]
    CodeNode,
}

impl Related<super::code_nodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CodeNode.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
