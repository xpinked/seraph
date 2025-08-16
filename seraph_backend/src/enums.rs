use std::fmt::Display;

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, DeriveActiveEnum, EnumIter)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "code_language")]
#[serde(rename_all = "lowercase")]
pub enum CodeLanguage {
    #[sea_orm(string_value = "python")]
    Python,
    #[sea_orm(string_value = "javascript")]
    JavaScript,
}

impl Display for CodeLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeLanguage::Python => write!(f, "python"),
            CodeLanguage::JavaScript => write!(f, "javascript"),
        }
    }
}

impl CodeLanguage {
    pub fn get_extension(&self) -> &str {
        match self {
            CodeLanguage::Python => "py",
            CodeLanguage::JavaScript => "js",
        }
    }

    pub fn get_image_name(&self) -> &str {
        match self {
            CodeLanguage::Python => "python:3.12",
            CodeLanguage::JavaScript => "node:latest",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, DeriveActiveEnum, EnumIter)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "output_type")]
#[serde(rename_all = "lowercase")]
pub enum OutputType {
    #[sea_orm(string_value = "string")]
    String,
    #[sea_orm(string_value = "number")]
    Number,
    #[sea_orm(string_value = "boolean")]
    Boolean,
    #[sea_orm(string_value = "array")]
    Array,
    #[sea_orm(string_value = "object")]
    Object,
    #[sea_orm(string_value = "not_output")]
    NoOutput, // This can be used for nodes that do not produce an output
}

impl Display for OutputType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputType::String => write!(f, "string"),
            OutputType::Number => write!(f, "number"),
            OutputType::Boolean => write!(f, "boolean"),
            OutputType::Array => write!(f, "array"),
            OutputType::Object => write!(f, "object"),
            OutputType::NoOutput => write!(f, "not_output"),
        }
    }
}
