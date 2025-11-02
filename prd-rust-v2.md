# Product Requirements Document: Rapid MCP Server (Rust Implementation)

**Version:** 2.0 (Rust-first rewrite)
**Date:** 2025-10-30
**Author:** John Junkins (@macjunkins)
**Status:** Approved for Implementation

---

## Executive Summary

Build a high-performance Model Context Protocol (MCP) server in Rust that exposes RAPID workflow commands to all MCP-compatible AI clients (Claude Code, GitHub Copilot, Codex CLI, etc.). This is a Rust-first implementation aligned with this repository’s structure, designed for fast startup, robust validation, and safe integration with the GitHub CLI.

Key Goals:
- Single source of truth for RAPID workflow commands (YAML)
- Universal AI client compatibility via MCP protocol
- High-performance, safe, minimal-dependency Rust implementation
- Foundation for RapidOS integration

---

## Background & Motivation

### Current Problem
- RAPID commands (≈25) live as Markdown workflows; not universally consumable by different AI tools.
- Need a single MCP server that exposes these workflows to any MCP-capable client.

### Solution Overview
Build an MCP server in Rust that:
1. Loads command definitions from YAML files.
2. Exposes them as MCP tools with JSON Schema input definitions.
3. Validates all parameters rigorously (never trust AI input).
4. Integrates with GitHub CLI (`gh`) for operations without spawning shells.
5. Optionally offers an HTTP bridge post-MVP.

### Why Rust?
- Performance and predictable latency.
- Strong type system, memory safety without GC.
- Mature ecosystem for serialization (`serde`), YAML, and templating (`handlebars`).
- Excellent cross-compilation and distribution options.

---

## Technical Architecture

### System Overview
```
AI Clients (Claude, Copilot, etc.)
        │ JSON-RPC 2.0 over stdio (MCP)
        ▼
Rapid MCP Server (Rust)
  - YAML Command Loader
  - Command Registry
  - Parameter Validation
  - GitHub CLI Wrapper
  - MCP Dispatcher
        │
        ▼
External Tools
  - gh CLI
  - git
  - filesystem
```

### Repository Layout (Rust)
```
rapid-mcp-server-rust/
├── src/
│   ├── main.rs                # Entry point, stdio loop
│   ├── mcp/
│   │   ├── types.rs           # JSON-RPC + MCP types
│   │   ├── server.rs          # Handlers: initialize, tools/list, tools/call
│   │   └── mod.rs
│   ├── command/
│   │   ├── types.rs           # YAML schema structs
│   │   ├── loader.rs          # YAML loader + registry
│   │   └── mod.rs
│   ├── validation/            # Validation checks (MVP + optional regex)
│   └── github/
│       ├── cli.rs             # gh wrapper (no shell)
│       └── mod.rs
├── commands/                  # YAML command definitions (symlink or copy)
├── benches/                   # Startup & dispatch benchmarks
├── docs/
│   └── commands/              # Human-readable command docs (optional)
├── Cargo.toml
└── README.md
```

### Core Crates
- serde / serde_json / serde_yaml — serialization
- handlebars — template engine for prompt rendering
- anyhow / thiserror — error handling
- tokio (optional for async stdio; MVP can use sync stdio)
- regex (optional, behind feature flag `regex_validation`)

---

## YAML Command Schema

We adopt the same logical schema as the previous document, implemented as Rust structs with `serde` derives.

```yaml
name: string
version: string
description: string
category: string
parameters:
  - name: string
    type: string            # "string" | "integer" | "number" | "boolean" | "array" | "object"
    required: boolean
    description: string
    default: any
    validation:
      pattern: string       # for documentation; evaluated if regex feature enabled
      min: number
      max: number
      min_length: number
      max_length: number
      allowed_values: []
examples:
  - description: string
    args: object
prompt: |
  Multi-line text with template placeholders like {{parameter}}.
  Optionally {{#if}} / {{#each}} post-MVP (helpers enabled later).
metadata:
  os_integration:
    requires_git: boolean
    requires_gh_cli: boolean
    system_permissions: []
    sandbox_profile: string
```

Rust model sketch:
```
pub struct Command { /* name, version, description, category, parameters, examples, prompt, metadata */ }
pub struct Parameter { /* name, type_, required, description, default, validation */ }
pub struct Validation { /* pattern, min, max, min_length, max_length, allowed_values */ }
```

Schema → MCP `tools/list` mapping:
- Build JSON Schema object (`type: object`, `properties`, `required`) from `parameters`.
- Preserve `pattern` as a string; enforce only when `regex_validation` feature is on.

---

## MCP Protocol Implementation

Transport: JSON-RPC 2.0 over stdio. MVP uses line-delimited JSON frames; can add Content-Length framing later for robustness.

Required methods:
1) `initialize` — Return protocol version, capabilities, server info.
2) `notifications/initialized` — No-op ack (no response).
3) `tools/list` — Enumerate tools with input schemas from YAML.
4) `tools/call` — Validate args, render prompt, optionally interact with `gh`, return `content: [{ type: "text", text: ... }]`.

Types are defined in `mcp/types.rs` and serialized with `serde`.

---

## Parameter Validation System

Never trust AI input. Validation is performed before any command execution.

MVP rules (no regex dependency required):
- Type checks: string, integer, number, boolean, array, object (basic shape).
- String checks: min/max length, allowed values; optional owner/repo validator.
- Number checks: min/max.
- Array checks: min/max items, optional uniqueness.
- GitHub-specific checks (hand-coded):
  - Repository `owner/repo`: two segments, allowed chars `[A-Za-z0-9_-]`, max len 100, single `/`.
  - Branch name: allowed chars `[A-Za-z0-9/_-]`, max len 255.
  - Issue number: positive integer only.

Post-MVP (feature `regex_validation`):
- Enable `regex` crate and honor `pattern` rules from YAML for string parameters.

Error responses:
- Use `-32602 Invalid params` with `data.validation_errors: [{ field, error_type, message }]`.

---

## GitHub CLI Integration

- Execution: `std::process::Command` with argument vectors only.
- Never spawn via shell; pass titles/bodies via files/stdin to avoid quoting issues.
- Typical flow for `gh-work`:
  - `gh issue view <num> --json title,body,labels,milestone,state,number,assignees` (plus `-R owner/repo` when not in a repo).
  - Parse stdout as JSON with `serde_json`.

Wrapper prototype:
```
pub struct GhOutput { pub stdout: String, pub json: serde_json::Value }

pub fn exec_gh(args: &[&str]) -> anyhow::Result<GhOutput> { /* spawn + parse */ }
```

Error handling:
- Handle not-installed gh, auth failures, network errors, non-JSON output.
- Surface clear errors back through MCP error envelopes.

---

## Implementation Milestones (Rust)

### Milestone 1: Foundation (Weeks 1–2)
Goal: Prove Rust + MCP + YAML architecture end-to-end.

Tasks:
- Initialize crate dependencies; feature flag `regex_validation` off by default.
- Implement `command::types` and `command::loader` (load 3 commands).
- Implement MCP `initialize`, `notifications/initialized`, `tools/list`.
- Skeleton `tools/call` that validates and renders a simple prompt (no `gh`).

Deliverables:
- `cargo run` starts server; `tools/list` shows 3 commands.
- Validation rejects invalid input with structured errors.

### Milestone 2: MVP Commands (Week 3)
Goal: Full end-to-end execution of 3 commands.

Tasks:
- Implement `validation` module with MVP rules.
- Implement GitHub wrapper and wire `gh-work`.
- Implement `sanity-check` (local checks, no `gh`).
- Implement `create-issue` (staged prompt → approval → `gh issue create` with body via file/stdin).

Deliverables:
- 3 MVP commands work via a real MCP client (e.g., Claude Code).

### Milestone 3: Full Command Port (Weeks 4–6)
Goal: Convert the remaining ~22 commands and robust error handling.

Tasks:
- Batch convert YAMLs; category-specific validation and preflight checks.
- Logging to `/tmp/rapid-mcp.log` (opt-in via env var).
- Hot reload optional (dev-only) — defer unless necessary.
- Unit + integration tests; performance baseline.

Deliverables:
- All commands load and execute.
- Tests passing; clear error propagation.

### Milestone 4: Optional HTTP Bridge (Week 7)
- Only if needed for non-MCP clients.
- Minimal `axum` endpoint `POST /run` bridging to the dispatcher.
- Config via env var: port, commands dir, log file path.

### Milestone 5: Distribution & RapidOS Prep (Week 8)
- Release builds; MUSL for Linux, macOS x86_64 + ARM64, Windows MSVC.
- Package scripts; optional systemd/launchd files.
- YAML metadata `os_integration` finalized.

---

## Testing Strategy

- Unit tests: loader, validator, MCP types/serde, GitHub wrapper (mock mode).
- Integration tests: end-to-end MCP flow; `gh` calls behind a mock feature for CI.
- Performance: Criterion benchmarks for startup time and `tools/list` latency.
- Compatibility: Test with Claude Code, and at least one more MCP client.

---

## Performance & Security Targets

Performance:
- Startup: < 100 ms listing tools (preloaded YAML).
- Command latency (non-`gh`): < 100–150 ms.
- `gh` commands: network-bound; aim for efficient JSON parsing.

Security:
- No shell invocation; argument vectors only.
- Strict validation of all user inputs.
- Size/time limits on stdio frames to avoid abuse.

---

## Risks & Mitigations (Rust-specific)

- JSON-RPC framing over stdio is brittle → Start with line-delimited JSON; add Content-Length framing if needed.
- Cross-platform stdio differences → CI on Windows/macOS/Linux for MCP handshake.
- Regex crate size/perf → Keep behind feature flag; default to hand-rolled validators.
- Static linking (MUSL) → Prefer pure-Rust deps; avoid crates needing OpenSSL.

---

## Open Questions

1. Do we require `tokio` for MVP, or keep synchronous stdio initially and add async later?
2. Do we want hot reload of YAML in dev mode for rapid iterations?
3. Do we accept `regex` in default builds, or keep it feature-gated?

---

## Next Actions

1. Confirm dependency set and feature flags (`regex_validation`, `mock_gh`).
2. Finalize Rust structs in `command::types` and implement the YAML loader.
3. Implement MCP `initialize` + `tools/list` and prove round-trip with a client.
4. Port `sanity-check.yaml` and `gh-work.yaml` into `commands/` (or confirm symlink).
5. Implement validator skeleton and `tools/call` for `sanity-check`.
6. Add `gh` wrapper and integrate `gh-work`.

---

## Changelog

- v2.0 (2025-10-30): Full Rust-first PRD rewrite; architecture, milestones, dependencies, and risks updated for Rust implementation.
