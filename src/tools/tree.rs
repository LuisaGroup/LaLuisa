// this module implements the tree command, which lists the contents of a directory.

use crate::tools::Tool;
use anyhow::Result;
use tool_protocol::{ToolArgument, ToolProtocol, ToolSchema, get_schema};
use tool_protocol_derive::ToolProtocol;

#[derive(ToolProtocol)]
#[tool_protocol(
    name = "tree",
    help = "Lists the contents of a directory, optionally with the given recursive depth."
)]
struct TreeToolProtocol {
    #[tool_protocol(
        help = "The path to the directory to list.",
        example = "/path/to/directory",
        required = true,
        default = "."
    )]
    path: String,

    #[tool_protocol(
        help = "The maximum depth to recurse into the directory (will be clamped to [1, 3] to avoid flushing).",
        default = 2,
        example = 1
    )]
    depth: u32,
}

pub(crate) struct Tree {
    schema: ToolSchema,
}

impl Tool for Tree {
    fn create() -> Box<dyn Tool> {
        Box::new(Self {
            schema: get_schema::<TreeToolProtocol>(),
        })
    }

    fn get_schema(&self) -> &ToolSchema {
        &self.schema
    }

    fn invoke(&self, args: &serde_json::Value) -> Result<String> {
        Err(anyhow::anyhow!("Not implemented"))
    }
}
