// this module implements the read command, which reads the contents of a text file.

use crate::tools::{Tool, ToolSchema};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use tool_protocol::ToolProtocol;
use tool_protocol::{ToolArgument, canonicalize_tool_args, create_schema, parse_args};
use tool_protocol_derive::{ToolProtocol, tool};

#[derive(ToolProtocol, Serialize, Deserialize)]
#[tool_protocol(name = "read", help = "Reads the contents of a text file.")]
struct ReadToolProtocol {
    #[tool_protocol(
        help = "The path to the file to read.",
        example = "/path/to/file",
        required
    )]
    path: String,
}

#[tool(ReadToolProtocol)]
pub struct Read {
    schema: ToolSchema,
}

impl Read {
    pub fn create() -> Rc<RefCell<dyn Tool>> {
        Rc::new(RefCell::new(Self {
            schema: create_schema::<ReadToolProtocol>(),
        }))
    }

    pub fn invoke(&mut self, args: ReadToolProtocol) -> Result<String> {
        todo!()
    }
}
