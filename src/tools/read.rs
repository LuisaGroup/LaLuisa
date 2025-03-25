// this module implements the read command, which reads the contents of a text file.

use crate::tools::{Tool, ToolSchema};
use anyhow::Result;
use tool_protocol::ToolProtocol;
use tool_protocol::{ToolArgument, get_schema};
use tool_protocol_derive::ToolProtocol;

#[derive(ToolProtocol)]
#[tool_protocol(name = "read", help = "Reads the contents of a text file.")]
struct ReadToolProtocol {
    #[tool_protocol(
        help = "The path to the file to read.",
        example = "/path/to/file",
        required
    )]
    path: String,
}

pub(crate) struct Read {
    schema: ToolSchema,
}

impl Tool for Read {
    fn create() -> Box<dyn Tool> {
        Box::new(Self {
            schema: get_schema::<ReadToolProtocol>(),
        })
    }

    fn get_schema(&self) -> &ToolSchema {
        &self.schema
    }

    fn invoke(&self, args: &serde_json::Value) -> Result<String> {
        let path = args["path"].as_str().unwrap();
        let content = std::fs::read_to_string(path)?;
        Ok(content)
    }
}
