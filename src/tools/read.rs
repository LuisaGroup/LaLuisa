// this module implements the read command, which reads the contents of a text file.

use tool_protocol::ToolArgument;
use tool_protocol::ToolProtocol;
use crate::tools::{Tool, ToolSchema};
use anyhow::Result;
use tool_protocol_derive::ToolProtocol;

#[derive(ToolProtocol)]
#[tool_protocol(help = "Reads the contents of a text file.")]
struct ReadToolProtocol {
    #[tool_protocol(
        help = "The path to the file to read.",
        example = "/path/to/file",
        required = true
    )]
    path: String,
}

pub(crate) struct Read {
    // desc: ToolSchema,
}

impl Read {
    pub fn new() -> Self {
        Self {}
    }
}

impl Tool for Read {
    fn get_schema(&self) -> ToolSchema {
        ReadToolProtocol::get_schema()
    }

    fn invoke(&self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let path = args["path"].as_str().unwrap();
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::json!(content))
    }
}
