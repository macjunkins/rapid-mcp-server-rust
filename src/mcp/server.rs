use anyhow::Result;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

use super::types::*;
use crate::command::loader::CommandRegistry;

pub struct McpServer {
    registry: CommandRegistry,
}

impl McpServer {
    pub fn new(registry: CommandRegistry) -> Self {
        Self { registry }
    }

    pub fn run(&self) -> Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let line = line?;

            if let Ok(request) = serde_json::from_str::<JsonRpcRequest>(&line) {
                let response = self.handle_request(&request);

                let response_json = serde_json::to_string(&response)?;
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
            }
        }

        Ok(())
    }

    fn handle_request(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(),
            "tools/list" => self.handle_tools_list(),
            "tools/call" => self.handle_tools_call(request.params.as_ref()),
            _ => Err(format!("Unknown method: {}", request.method)),
        };

        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(value),
                error: None,
            },
            Err(msg) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: msg,
                }),
            },
        }
    }

    fn handle_initialize(&self) -> Result<Value, String> {
        Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "rapid-mcp-server-rust",
                "version": "0.1.0"
            }
        }))
    }

    fn handle_tools_list(&self) -> Result<Value, String> {
        let tools: Vec<Value> = self.registry.list()
            .iter()
            .map(|cmd| {
                json!({
                    "name": cmd.name,
                    "description": cmd.description,
                    "inputSchema": {
                        "type": "object",
                        "properties": {},
                        "required": []
                    }
                })
            })
            .collect();

        Ok(json!({ "tools": tools }))
    }

    fn handle_tools_call(&self, params: Option<&Value>) -> Result<Value, String> {
        let params = params.ok_or("Missing params")?;
        let tool_name = params["name"].as_str().ok_or("Missing tool name")?;

        let command = self.registry.get(tool_name)
            .ok_or_else(|| format!("Unknown tool: {}", tool_name))?;

        // For MVP, just return the prompt template
        // TODO: Substitute parameters using handlebars

        Ok(json!({
            "content": [{
                "type": "text",
                "text": command.prompt
            }]
        }))
    }
}
