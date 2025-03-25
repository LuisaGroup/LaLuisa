use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolArgument {
    pub name: String,
    pub help: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub required: bool,
    pub default: serde_json::Value,
    pub example: serde_json::Value,
}

impl ToolArgument {
    pub fn get_help(&self) -> serde_json::Value {
        let mut help = serde_json::Map::new();
        help.insert(
            "type".to_string(),
            serde_json::Value::String(self.type_.clone()),
        );
        help.insert(
            "required".to_string(),
            serde_json::Value::Bool(self.required),
        );
        if !self.default.is_null() {
            help.insert("default".to_string(), self.default.clone());
        }
        if !self.example.is_null() {
            help.insert("example".to_string(), self.example.clone());
        }
        serde_json::Value::Object(help)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub help: String,
    pub arguments: Vec<ToolArgument>,
}

impl ToolSchema {
    pub fn get_example(&self) -> serde_json::Value {
        let mut example = serde_json::Map::new();
        for arg in &self.arguments {
            let value = if !arg.example.is_null() {
                arg.example.clone()
            } else if !arg.default.is_null() {
                arg.default.clone()
            } else {
                assert!(!arg.required);
                serde_json::Value::Null
            };
            if !value.is_null() {
                example.insert(arg.name.clone(), value);
            }
        }
        serde_json::Value::Object(example)
    }

    pub fn get_help(&self) -> serde_json::Value {
        let mut help = serde_json::Map::new();
        help.insert("example".to_string(), self.get_example());
        let mut args = serde_json::Map::new();
        for arg in &self.arguments {
            args.insert(arg.name.clone(), arg.get_help());
        }
        help.insert("arguments".to_string(), serde_json::Value::Object(args));
        serde_json::Value::Object(help)
    }
}

pub fn canonicalize_tool_args(
    schema: &ToolSchema,
    args: &serde_json::Value,
) -> Result<serde_json::Value> {
    let mut cargs = serde_json::Map::new();
    for arg in &schema.arguments {
        if let Some(value) = args.get(&arg.name) {
            cargs.insert(arg.name.clone(), value.clone());
        } else if !arg.default.is_null() {
            cargs.insert(arg.name.clone(), arg.default.clone());
        } else if arg.required {
            return Err(anyhow::anyhow!("Missing required argument: {}", arg.name));
        }
    }
    Ok(serde_json::Value::Object(cargs))
}

pub trait ToolProtocol<T> {
    fn create_schema() -> ToolSchema;
    fn parse_args(schema: &ToolSchema, args: &serde_json::Value) -> Result<T>
    where
        T: Sized;
}

pub fn create_schema<T: ToolProtocol<T>>() -> ToolSchema {
    T::create_schema()
}

pub fn parse_args<T: ToolProtocol<T>>(schema: &ToolSchema, args: &serde_json::Value) -> Result<T> {
    T::parse_args(schema, args)
}

pub trait Tool {
    fn get_schema(&self) -> &ToolSchema;
    fn invoke(&mut self, args: &serde_json::Value) -> Result<String>;
}
