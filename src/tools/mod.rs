mod read;
mod tree;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;

use tool_protocol::ToolSchema;

pub trait Tool {
    fn get_schema(&self) -> ToolSchema;
    fn invoke(&self, args: &serde_json::Value) -> Result<serde_json::Value>;
}

pub fn all_tools() -> Vec<Box<dyn Tool>> {
    let tools: Vec<Box<dyn Tool>> = vec![Box::new(tree::Tree::new()), Box::new(read::Read::new())];
    tools
}
