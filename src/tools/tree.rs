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
        help = "The maximum depth to recurse into the directory.",
        default = 4,
        example = 3
    )]
    depth: u32,
}

#[tool(TreeToolProtocol)]
pub struct Tree {
    schema: ToolSchema,
}

fn list_directory_tree(path: &str, depth: u32) -> Result<serde_json::Value> {
    use std::fs;
    use std::path::Path;

    fn build_tree(path: &Path, current_depth: u32, max_depth: u32) -> Result<serde_json::Value> {
        if current_depth > max_depth {
            return Ok(serde_json::Value::Null);
        }

        let mut node = serde_json::Map::new();
        node.insert(
            "name".to_string(),
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string()
                .into(),
        );
        node.insert(
            "path".to_string(),
            path.to_string_lossy().to_string().into(),
        );
        node.insert(
            "type".to_string(),
            if path.is_dir() { "directory" } else { "file" }.into(),
        );

        if path.is_dir() {
            let mut children = Vec::new();
            let entries = fs::read_dir(path)?;

            for entry in entries {
                let entry = entry?;
                let child_path = entry.path();
                if let Ok(child) = build_tree(&child_path, current_depth + 1, max_depth) {
                    if !child.is_null() {
                        children.push(child);
                    }
                }
            }

            if !children.is_empty() {
                node.insert("children".to_string(), children.into());
            }
        }

        Ok(serde_json::Value::Object(node))
    }

    let path = Path::new(path)
        .canonicalize()
        .map_err(|e| anyhow::anyhow!("Failed to canonicalize path: {}", e))?;

    if !path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", path.display()));
    }

    if !path.is_dir() {
        return Err(anyhow::anyhow!(
            "Path is not a directory: {}",
            path.display()
        ));
    }

    build_tree(&path, 0, depth)
}

impl Tree {
    pub fn create() -> Box<RefCell<dyn Tool>> {
        Box::new(RefCell::new(Self {
            schema: create_schema::<TreeToolProtocol>(),
        }))
    }

    fn invoke(&mut self, args: TreeToolProtocol) -> Result<String> {
        serde_json::to_string_pretty(&list_directory_tree(&args.path, args.depth)?)
            .map_err(Into::into)
    }
}
