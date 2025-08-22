use crate::enums::{CodeLanguage, OutputType};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use tempfile::TempPath;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "code_nodes")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,

    pub name: String,
    pub function_name: String,
    pub code: String,
    pub output_name: String,
    pub output_type: OutputType,
    pub language: CodeLanguage,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::code_result::Entity")]
    CodeResults,
}

impl Related<super::code_result::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CodeResults.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub fn alter_code(node: &Model) -> String {
    use unescape::unescape;

    let _deserialized_code: String = serde_json::from_str(&node.code).unwrap();
    let unescaped_code = unescape(&_deserialized_code).unwrap();
    let code_content = unescaped_code.trim_matches(char::from(0));

    return code_content.to_string();
}

pub struct Command {
    command: Vec<String>,
}
impl Command {
    pub fn new() -> Self {
        Self { command: vec![] }
    }

    pub fn add(&mut self, arg: &str) -> &mut Self {
        self.command.push(arg.to_string());
        self
    }

    pub fn to_vec(&self) -> Vec<String> {
        self.command.clone()
    }
}

impl Model {
    pub fn get_command(&self, args: &Vec<String>, dependencies: Option<&Vec<String>>) -> Vec<String> {
        match self.language {
            CodeLanguage::Python => {
                let mut command = Command::new();
                command.add("uv").add("run").add("-q");

                if let Some(deps) = dependencies {
                    deps.into_iter().for_each(|dep| {
                        command.add("--with").add(&dep);
                    });
                }

                command.add("main.py").add(&self.name).add(&self.function_name);

                args.into_iter().for_each(|arg| {
                    command.add(&arg);
                });

                command.to_vec()
            }
            CodeLanguage::JavaScript => todo!(),
        }
    }

    pub async fn to_tar(&self) -> TempPath {
        use tokio::fs::File;
        use tokio_tar as tar;

        let altered_code = alter_code(&self);
        let file_path = tempfile::NamedTempFile::new().unwrap();
        tokio::fs::write(&file_path, altered_code).await.expect("Failed to write code to file");

        let tar_path = tempfile::Builder::new().suffix(".tar").tempfile().unwrap().into_temp_path();
        let tar_file = tokio::fs::File::create(&tar_path).await.unwrap();
        let mut tar_builder = tar::Builder::new(tar_file);
        tar_builder
            .append_file(
                format!("{}.{}", &self.name, &self.language.get_extension()),
                &mut File::open(&file_path).await.unwrap(),
            )
            .await
            .unwrap();

        tar_path
    }
}
