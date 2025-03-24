mod read;
mod tree;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
pub struct Schema {
    name: String,
    brief: String,
    args: serde_json::Value,
    returns: serde_json::Value,
}

pub trait Tool {
    fn schema(&self) -> &Schema;
    fn invoke(&self, args: &serde_json::Value) -> Result<serde_json::Value>;
}

pub fn all_tools() -> Vec<Box<dyn Tool>> {
    let tools: Vec<Box<dyn Tool>> = vec![Box::new(tree::Tree::new()), Box::new(read::Read::new())];
    tools
}
