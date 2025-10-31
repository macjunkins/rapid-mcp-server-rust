use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct Command {
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    pub prompt: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
    #[serde(default)]
    pub required: bool,
    pub default: Option<String>,
}
