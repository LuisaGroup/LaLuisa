// this module implements the tree command, which lists the contents of a directory.

use crate::tools::{Schema, Tool};
use anyhow::Result;

pub(crate) struct Tree {
    desc: Schema,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            desc: Schema {
                name: "tree".to_string(),
                brief: "List the contents of a directory with a tree structure".to_string(),
                args: serde_json::json!({
                    "path": {
                        "type": "string",
                        "description": "The path to the directory",
                        "required": true,
                    },
                    "depth": {
                        "type": "integer",
                        "description": "The maximum depth of the tree (default: 0, unlimited)",
                        "default": 0,
                    }
                }),
                returns: serde_json::json!({
                    "type": "object",
                    "description": "The directory tree structure as a JSON object",
                }),
            },
        }
    }
}

impl Tool for Tree {
    fn schema(&self) -> &Schema {
        &self.desc
    }

    fn invoke(&self, args: &serde_json::Value) -> Result<serde_json::Value> {
        Err(anyhow::anyhow!("Not implemented"))
    }
}
