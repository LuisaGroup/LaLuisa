mod patch;
mod read;
mod tree;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::iter::Map;

use tool_protocol::ToolSchema;

pub trait Tool {
    fn create() -> Box<dyn Tool>
    where
        Self: Sized;
    fn get_schema(&self) -> &ToolSchema;
    fn invoke(&self, args: &serde_json::Value) -> Result<String>;
}

fn all_tools() -> Vec<Box<dyn Tool>> {
    vec![read::Read::create(), tree::Tree::create()]
}

pub struct ToolSet {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolSet {
    pub fn new() -> Self {
        Self {
            tools: all_tools()
                .into_iter()
                .map(|tool| (tool.get_schema().name.clone(), tool))
                .collect(),
        }
    }

    pub fn get_tools(&self) -> &HashMap<String, Box<dyn Tool>> {
        &self.tools
    }

    pub fn get_help(&self) -> serde_json::Value {
        let mut help = serde_json::Map::new();
        for (name, tool) in &self.tools {
            help.insert(name.clone(), tool.get_schema().get_help());
        }
        serde_json::Value::Object(help)
    }

    pub fn invoke(&self, name: &str, args: &serde_json::Value) -> Result<String> {
        if let Some(tool) = self.tools.get(name) {
            tool.invoke(args)
        } else {
            Err(anyhow::anyhow!("Unknown tool: {}", name))
        }
    }
}
