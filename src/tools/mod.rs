mod patch;
mod read;
mod tree;

pub use read::Read;
use std::cell::RefCell;
pub use tree::Tree;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use tool_protocol::{Tool, ToolSchema};

pub fn create_all_tools() -> Vec<Box<RefCell<dyn Tool>>> {
    vec![Read::create(), Tree::create()]
}

#[derive(Default)]
pub struct ToolSet {
    tools: HashMap<String, Box<RefCell<dyn Tool>>>,
}

impl ToolSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_tool(&mut self, tool: Box<RefCell<dyn Tool>>) {
        let name = tool.borrow().get_schema().name.clone();
        self.tools.insert(name, tool);
    }

    pub fn register_tools<T: IntoIterator<Item = Box<RefCell<dyn Tool>>>>(&mut self, tools: T) {
        for tool in tools {
            self.register_tool(tool);
        }
    }

    pub fn get_tools(&self) -> &HashMap<String, Box<RefCell<dyn Tool>>> {
        &self.tools
    }

    pub fn get_help(&self) -> serde_json::Value {
        let mut help = serde_json::Map::new();
        for (name, tool) in &self.tools {
            help.insert(name.clone(), tool.borrow().get_schema().get_help());
        }
        serde_json::Value::Object(help)
    }

    pub fn invoke(&self, name: &str, args: &serde_json::Value) -> Result<String> {
        if let Some(tool) = self.tools.get(name) {
            tool.try_borrow_mut()?.invoke(args)
        } else {
            Err(anyhow::anyhow!("Unknown tool: {}", name))
        }
    }
}
