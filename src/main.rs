mod mcp;
mod command;

use anyhow::Result;
use command::loader::CommandRegistry;
use mcp::server::McpServer;

fn main() -> Result<()> {
    eprintln!("Starting rapid-mcp-server-rust...");

    // Load commands
    let mut registry = CommandRegistry::new();
    registry.load_from_dir("commands")?;

    eprintln!("Loaded {} commands", registry.list().len());

    // Start MCP server
    let server = McpServer::new(registry);
    server.run()?;

    Ok(())
}
