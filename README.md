# Rapid MCP Server - Rust Implementation

A high-performance Model Context Protocol (MCP) server implementation in Rust.

## Overview

This is the Rust implementation of the Rapid MCP Server, designed for minimal startup time and maximum efficiency. It reads YAML-based command definitions from the [rapid-mcp-commands](https://github.com/macjunkins/rapid-mcp-commands) repository and exposes them via the MCP protocol.

## Project Status

🚧 **Phase 1: MVP Development** - Setting up foundational structure for Rust vs Go comparison.

## Architecture

```
rapid-mcp-server-rust/
├── src/
│   ├── main.rs              # Entry point
│   ├── mcp/                 # MCP protocol implementation
│   │   ├── types.rs         # MCP type definitions
│   │   ├── server.rs        # Protocol handler
│   │   └── mod.rs
│   ├── command/             # Command handling
│   │   ├── types.rs         # Command schema
│   │   ├── loader.rs        # YAML loader
│   │   └── mod.rs
│   ├── validation/          # Parameter validation (future)
│   └── github/              # GitHub API integration (future)
├── commands/                # Symlink to rapid-mcp-commands
├── benches/                 # Performance benchmarks
└── Cargo.toml
```

## Dependencies

- **serde/serde_json/serde_yaml** - Serialization
- **handlebars** - Template engine for parameter substitution
- **anyhow/thiserror** - Error handling
- **tokio** - Async runtime (minimal features)

## Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run
cargo run --release
```

## Testing

```bash
# Run tests
cargo test

# Run benchmarks
cargo bench

# Check startup time
time ./target/release/rapid-mcp-server < /dev/null
```

## MCP Protocol

This server implements the Model Context Protocol v2024-11-05:

- **initialize** - Returns server capabilities
- **tools/list** - Lists available commands
- **tools/call** - Executes a command with parameters

## Related Projects

- [rapid-mcp-commands](https://github.com/macjunkins/rapid-mcp-commands) - Shared command definitions
- [rapid-mcp-server-go](https://github.com/macjunkins/rapid-mcp-server-go) - Go implementation (for comparison)

## License

MIT License
