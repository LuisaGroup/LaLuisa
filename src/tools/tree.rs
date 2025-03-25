// this module implements the tree command, which lists the contents of a directory.

use crate::tools::Tool;
use anyhow::Result;
use tool_protocol::{get_schema, ToolArgument, ToolProtocol, ToolSchema};
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
        help = "The maximum depth to recurse into the directory.",
        default = 0,
        example = 2
    )]
    depth: u32,
}

pub(crate) struct Tree;

impl Tree {
    pub fn new() -> Self {
        Self {}
    }
}

impl Tool for Tree {
    fn get_schema(&self) -> ToolSchema {
        get_schema::<TreeToolProtocol>()
    }

    fn invoke(&self, args: &serde_json::Value) -> Result<serde_json::Value> {
        Err(anyhow::anyhow!("Not implemented"))
    }
}
