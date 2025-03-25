// this module implements the tree command, which lists the contents of a directory.

use crate::tools::Tool;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use tool_protocol::{
    ToolArgument, ToolProtocol, ToolSchema, canonicalize_tool_args, create_schema, parse_args,
};
use tool_protocol_derive::{ToolProtocol, tool};

#[derive(ToolProtocol, Serialize, Deserialize, Debug)]
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

#[tool(TreeToolProtocol)]
pub struct Tree {
    schema: ToolSchema,
}

impl Tree {
    pub fn create() -> Box<RefCell<dyn Tool>> {
        Box::new(RefCell::new(Self {
            schema: create_schema::<TreeToolProtocol>(),
        }))
    }

    pub fn invoke(&mut self, args: TreeToolProtocol) -> Result<String> {
        todo!()
    }
}
