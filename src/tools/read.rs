// this module implements the read command, which reads the contents of a text file.

use crate::tools::{Schema, Tool};
use anyhow::Result;

pub(crate) struct Read {
    desc: Schema,
}

impl Read {
    pub fn new() -> Self {
        Self {
            desc: Schema {
                name: "read".to_string(),
                brief: "Read the contents of a text file".to_string(),
                args: serde_json::json!({
                    "path": {
                        "type": "string",
                        "description": "The path to the text file",
                        "required": true,
                    }
                }),
                returns: serde_json::json!({
                    "type": "string",
                    "description": "The contents of the text file",
                }),
            },
        }
    }
}

impl Tool for Read {
    fn schema(&self) -> &Schema {
        &self.desc
    }

    fn invoke(&self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let path = args["path"].as_str().unwrap();
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::json!(content))
    }
}
