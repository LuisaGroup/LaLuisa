use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolArgument {
    pub name: String,
    pub help: String,
    pub type_: String,
    pub required: bool,
    pub default: serde_json::Value,
    pub example: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub help: String,
    pub arguments: Vec<ToolArgument>,
}

pub trait ToolProtocol {
    fn get_schema() -> ToolSchema;
}
