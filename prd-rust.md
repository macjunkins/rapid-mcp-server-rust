# Product Requirements Document: Rapid MCP Server (Zig Implementation)

**Version:** 1.1
**Date:** 2025-10-29 (Updated from 2025-10-28)
**Author:** John Junkins (@macjunkins)
**Status:** Revised - Reality Check Complete, Ready for Implementation

---

## Executive Summary

Build a high-performance Model Context Protocol (MCP) server in Zig that exposes RAPID workflow commands to all MCP-compatible AI clients (Claude Code, GitHub Copilot, Codex CLI, etc.). This replaces the planned Node.js implementation with Zig for better performance, zero runtime dependencies, and future RapidOS integration.

**Key Goals:**
- Single source of truth for RAPID workflow commands
- Universal AI client compatibility via MCP protocol
- High-performance, zero-dependency implementation in Zig
- Foundation for RapidOS "AI-first" operating system
- Community engagement with Zig ecosystem

---

## Background & Motivation

### Current Problem
- RAPID workflow commands (25 total) currently defined in `.claude/commands/*.md`
- Only accessible to Claude Code CLI
- Other AI tools (Copilot, Codex, etc.) cannot access these workflows
- Each tool requires separate integration work
- Node.js implementation planned but not optimal for system-level integration

### Proposed Solution
Build an MCP server in Zig that:
1. Reads workflow definitions from YAML files
2. Exposes them as MCP tools to any compatible AI client
3. Validates all parameters rigorously (never trust AI input)
4. Integrates with GitHub CLI for operations
5. Provides optional HTTP bridge for non-MCP clients

### Strategic Rationale: Why Zig?

**Technical Benefits:**
- ⚡ **Performance:** Blazing fast startup, critical for CLI tools and RapidOS
- 📦 **Zero dependencies:** Single static binary, no runtime required
- 🛡️ **Memory safety:** Explicit control without garbage collection overhead
- 🔧 **C interop:** Easy integration with system tools (gh CLI, git, etc.)
- 🚀 **Cross-platform:** Easy compilation for macOS, Linux, Windows

**Strategic Benefits:**
- 🎯 **RapidOS foundation:** Core system component written in Zig
- 👥 **Community play:** Attract Zig developers to RapidOS project
- 📈 **Differentiation:** "The AI-first distro built in Zig"
- 🔮 **Pioneer positioning:** Be early adopter of Zig for AI infrastructure

**The Meta-Strategy:**
> "Everyone tells me I shouldn't because no one else has done something in Zig. But how do we get things if no one ever does them?"

This project positions RapidOS as an innovative, performance-focused platform that embraces emerging technologies.

---

## Technical Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     AI Clients                              │
│  Claude Code │ Copilot │ Codex CLI │ VS Code │ Other MCP   │
└──────────────┬──────────┬───────────┬──────────┬────────────┘
               │          │           │          │
               └──────────┴───────────┴──────────┘
                          │ MCP Protocol (JSON-RPC over stdio)
               ┌──────────▼────────────────────┐
               │  Rapid MCP Server (Zig)      │
               │  - Command Registry          │
               │  - Parameter Validation      │
               │  - YAML Command Loader       │
               │  - GitHub CLI Wrapper        │
               └──────────┬────────────────────┘
                          │
               ┌──────────▼────────────────────┐
               │  External Tools               │
               │  - gh CLI (GitHub operations) │
               │  - git (repository context)   │
               │  - filesystem (docs, config)  │
               └───────────────────────────────┘
```

### Component Architecture

```
rapid-mcp-server/              # This repository
├── build.zig                   # Zig build configuration
├── build.zig.zon              # Package dependencies
├── src/
│   ├── main.zig               # Entry point, stdio event loop
│   ├── mcp.zig                # MCP protocol handlers (initialize, tools/list, tools/call)
│   ├── command.zig            # Command loader & registry (HashMap)
│   ├── validator.zig          # Parameter validation engine
│   ├── github.zig             # GitHub CLI wrapper (std.process)
│   ├── yaml_schema.zig        # YAML command schema definitions
│   └── http_bridge.zig        # Optional HTTP server (Milestone 4)
├── commands/                   # YAML command definitions (source of truth)
│   ├── sanity-check.yaml      # Convert from existing .md files
│   ├── gh-work.yaml
│   ├── create-issue.yaml
│   └── ... (22 more)
├── docs/
│   └── commands/              # Human-readable markdown docs (generated or archived .md)
├── test/
│   ├── unit/
│   └── integration/
├── prd.md                     # This document
└── README.md                  # Project overview
```

---

## YAML Command Schema

### Discovery from Prototype Conversion

**Key Finding:** Converted [sanity-check.md](commands/sanity-check.md) to [sanity-check.yaml](commands/sanity-check.yaml) - revealed template engine requirement.

**Challenges Found:**
1. **Optional flags** (`--strict`, `--reset`, `--scope`) → Mapped to boolean/string parameters ✅
2. **Conditional prompt logic** ("if strict mode", "if scope provided") → Requires template engine (Handlebars-style `{{#if}}`)
3. **Dynamic text insertion** → Simple `{{parameter}}` substitution works for MVP

**Implications:**
- **MVP Approach:** Simple string replacement (`{{parameter}}`) for basic commands
- **Post-MVP:** Add template engine library for complex conditionals (contradicts "zero dependencies")
- **Alternative:** Pre-render all conditional variants (explosion of YAML files)

**Decision Required:** Accept template engine dependency or simplify command workflows?

### Schema Structure

```yaml
name: string              # Unique tool identifier (e.g., "gh-work")
version: string           # Semantic version (e.g., "1.0.0")
description: string       # Brief tool description for AI (1-2 sentences)
category: string          # Grouping: github, qa, documentation, workflow, utility

parameters:               # Array of parameter definitions
  - name: string          # Parameter identifier
    type: string          # "string" | "integer" | "number" | "boolean" | "array" | "object"
    required: boolean     # Is parameter mandatory?
    description: string   # Help text for AI
    default: any          # Optional default value
    validation:           # Validation rules (optional)
      pattern: string     # Regex pattern (string type)
      min: number         # Minimum value (number type) or length (string type)
      max: number         # Maximum value (number type) or length (string type)
      min_length: number  # Minimum string length
      max_length: number  # Maximum string length
      allowed_values: []  # Enum-style allowed values

examples:                 # Usage examples for AI context
  - description: string   # Example description
    args: object          # Example arguments

prompt: |                 # Multi-line workflow instructions with template syntax
  The actual AI prompt text with template placeholders:
  - {{parameter}} - Simple substitution
  - {{#if parameter}}...{{/if}} - Conditional blocks (requires template engine)
  - {{#each array}}...{{/each}} - Iteration (requires template engine)

  **Note:** Complex commands may require Handlebars-style template engine.
  MVP may use simple string replacement only.

metadata:                 # Extensibility for RapidOS integration (Milestone 5)
  os_integration:
    requires_git: boolean
    requires_gh_cli: boolean
    system_permissions: []
    sandbox_profile: string
```

### Example: `gh-work.yaml`

```yaml
name: gh-work
version: "1.0.0"
description: "Work on a GitHub issue using investigate → plan → execute workflow"
category: github

parameters:
  - name: issue_number
    type: integer
    required: true
    description: "The GitHub issue number to work on"
    validation:
      min: 1
      max: 999999

  - name: repo
    type: string
    required: false
    description: "Repository in owner/repo format (optional if in git context)"
    validation:
      pattern: "^[\\w-]+/[\\w-]+$"
      max_length: 100

examples:
  - description: "Work on issue #6 in current repo"
    args:
      issue_number: 6

  - description: "Work on issue #42 in different repo"
    args:
      issue_number: 42
      repo: "macjunkins/other-project"

prompt: |
  You are working on a GitHub issue using the standardized investigate → plan → execute workflow.

  ## Instructions

  Follow this workflow strictly:

  ### 1. Investigate
  - Use `gh issue view {{issue_number}} --json title,body,labels,milestone,state,number,assignees` to fetch issue details
  - If the command fails with "not a git repository", ask the user to specify the repo: `gh issue view {{issue_number}} -R {{repo}} --json ...`
  - Present a clear summary of the issue including:
    - Title and number
    - Current state (open/closed)
    - Labels and milestone (if any)
    - Full description/body
    - Key tasks or requirements

  ### 2. Analyze
  - Review the issue body and identify all tasks
  - Note any dependencies on other issues
  - Identify success criteria
  - Check current state of codebase related to the issue

  ### 3. Plan
  Break down the work into specific, actionable steps including:
  - Investigation/research (if needed)
  - Implementation
  - Testing
  - Documentation updates

  ### 4. Present & Get Approval
  - Show the complete plan to the user
  - Explicitly ask: "Should I proceed with implementation?"
  - Wait for explicit approval

  ### 5. Execute
  - ONLY after receiving approval, begin implementation
  - Work through each step systematically
  - Follow all approval checkpoints

metadata:
  os_integration:
    requires_git: true
    requires_gh_cli: true
    system_permissions:
      - read:repo
      - write:issues
```

---

## MCP Protocol Implementation

### Required JSON-RPC Methods

The server must implement these MCP protocol methods:

#### 1. `initialize`
**Purpose:** Handshake and capability negotiation

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {},
    "clientInfo": {
      "name": "Claude Desktop",
      "version": "1.0.0"
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "tools": {}
    },
    "serverInfo": {
      "name": "rapid-mcp-server",
      "version": "1.0.0"
    }
  }
}
```

#### 2. `notifications/initialized`
**Purpose:** Acknowledge initialization complete

**Notification (no response):**
```json
{
  "jsonrpc": "2.0",
  "method": "notifications/initialized"
}
```

#### 3. `tools/list`
**Purpose:** List all available tools

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list"
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tools": [
      {
        "name": "gh-work",
        "description": "Work on a GitHub issue using investigate → plan → execute workflow",
        "inputSchema": {
          "type": "object",
          "properties": {
            "issue_number": {
              "type": "integer",
              "description": "The GitHub issue number to work on",
              "minimum": 1,
              "maximum": 999999
            },
            "repo": {
              "type": "string",
              "description": "Repository in owner/repo format",
              "pattern": "^[\\w-]+/[\\w-]+$",
              "maxLength": 100
            }
          },
          "required": ["issue_number"]
        }
      }
    ]
  }
}
```

#### 4. `tools/call`
**Purpose:** Execute a tool

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "gh-work",
    "arguments": {
      "issue_number": 42
    }
  }
}
```

**Response (Success):**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "You are working on GitHub issue #42...\n\n[Full prompt with workflow instructions]"
      }
    ]
  }
}
```

**Response (Validation Error):**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": {
      "validation_errors": [
        {
          "field": "issue_number",
          "error_type": "type_mismatch",
          "message": "Expected integer, got string"
        }
      ]
    }
  }
}
```

---

## Parameter Validation System

### Philosophy: Never Trust AI Input

**Why Validate?**
- AI clients can hallucinate values
- Prevent injection attacks (shell metacharacters in repo names)
- Type safety (string vs integer confusion)
- Resource protection (unbounded arrays, huge strings)
- Security boundary for RapidOS system integration

### Validation Rules

**Type Validation:**
- `string`, `integer`, `number`, `boolean`, `array`, `object`
- Strict type checking (no automatic coercion)

**String Validation:**
- `pattern`: Regex pattern matching
- `min_length` / `max_length`: Character count limits
- `allowed_values`: Enum-style whitelist

**Number Validation:**
- `min` / `max`: Range limits (inclusive)
- `multiple_of`: Divisibility check

**Array Validation:**
- `min_items` / `max_items`: Length limits
- `unique`: All elements must be unique
- `item_type`: Type of array elements

### Validation Flow

```
1. Parse tool call (JSON-RPC)
2. Lookup command schema (YAML)
3. Check required parameters present
4. Validate each parameter:
   a. Type validation
   b. Range/length validation
   c. Pattern validation
5. Return structured errors if invalid
6. Proceed to execution only if valid
```

### Implementation Strategy

**Milestone 1 (MVP):** Custom string-matching validators
- **No regex engine** - hand-coded character-by-character validation
- Simple pattern validators for common cases:
  - Repository: Check for `owner/name` format (alphanumeric + hyphens + underscores, single slash)
  - Branch names: Character-by-character check for allowed chars (alphanumeric + `/_-`)
  - Issue numbers: Integer parsing only (reject non-numeric)
  - Labels: Alphanumeric + hyphens + underscores
- **Limitation:** Cannot express complex patterns like email validation
- **Trade-off:** Zero dependencies vs limited validation expressiveness

**Milestone 2 (Post-MVP):** Add regex library if needed
- Options: PCRE binding, or Zig regex library if available
- **Note:** This contradicts "zero dependencies" goal - decision required before Milestone 2

**YAML Schema Pattern Field:**
- Patterns defined in YAML (e.g., `pattern: "^[a-zA-Z0-9_-]+/[a-zA-Z0-9_-]+$"`) are **documentation only** for MVP
- Actual validation uses hand-coded string matching, not regex evaluation
- Post-MVP: May implement regex evaluation if library added

---

## GitHub CLI Integration

All GitHub-related commands execute `gh` CLI and parse JSON output.

### GitHub Module API

```zig
pub fn execGh(
    allocator: Allocator,
    args: []const []const u8
) !GhResult {
    const result = try std.process.Child.exec(.{
        .allocator = allocator,
        .argv = args,
        .env_map = null,
    });

    if (result.term.Exited != 0) {
        return error.GhCommandFailed;
    }

    // Parse JSON response
    const parsed = try std.json.parseFromSlice(
        std.json.Value,
        allocator,
        result.stdout,
        .{}
    );

    return GhResult{ .json = parsed.value };
}
```

### Example Commands

**Fetch issue:**
```bash
gh issue view 42 --json title,body,labels,milestone,state,assignees
```

**Create issue:**
```bash
gh issue create --title "Title" --body "Body" --label "bug,enhancement"
```

**List issues:**
```bash
gh issue list --json number,title,state,labels --limit 10
```

### Security: Shell Injection Prevention

**Critical:** All parameters from AI clients must be sanitized before passing to `gh` CLI to prevent shell injection attacks.

**Validation Rules:**
- **Repository names:** Must match `^[a-zA-Z0-9_-]+/[a-zA-Z0-9_-]+$` (alphanumeric, hyphens, underscores, single slash)
- **Branch names:** Must match `^[a-zA-Z0-9/_-]+$` (no shell metacharacters: `;`, `|`, `&`, `$`, `` ` ``, `(`, `)`, `<`, `>`)
- **Issue numbers:** Must be positive integers only (no strings, no special characters)
- **Labels:** Alphanumeric, hyphens, underscores only (comma-separated list validated individually)
- **Titles/Bodies:** Pass via heredoc or stdin, never via command-line arguments directly

**Implementation Strategy:**
```zig
fn sanitizeRepoName(name: []const u8) ![]const u8 {
    // Validate format: owner/repo
    // Reject if contains shell metacharacters
    // Reject if contains path traversal (.., /)
    // Max length: 100 characters
}

fn sanitizeBranchName(name: []const u8) ![]const u8 {
    // Allow: alphanumeric, /, _, -
    // Reject: all other characters
    // Max length: 255 characters
}
```

**Example Attack Prevention:**
```zig
// UNSAFE: AI hallucinates malicious input
issue_number = "42; rm -rf /"  // ❌ Would execute: gh issue view 42; rm -rf /

// SAFE: Validation catches injection
const validated = try validateIssueNumber(issue_number);
// Returns error.InvalidIssueNumber - string contains non-numeric characters
```

### Error Handling

- Command not found (gh not installed)
- Authentication failure (gh auth login required)
- Network errors
- Invalid JSON response
- GitHub API rate limits
- **Shell injection attempts** (validation errors)

---

## Implementation Milestones

## Milestone 1: Foundation & Prototyping (MVP Core)

**Goal:** Prove Zig + MCP + YAML architecture works end-to-end

### 1.1 Project Setup
**Tasks:**
- [ ] Initialize Zig project in this repository
- [ ] Create `build.zig` with dependencies:
  - ZigJR (JSON-RPC 2.0) - `zig fetch --save https://github.com/williamw520/zigjr`
  - zig-yaml (YAML parsing) - `zig fetch --save https://github.com/kubkon/zig-yaml`
- [ ] Set up directory structure (src/, test/, docs/)
- [ ] Configure build targets (release-safe, release-fast, debug)

**Deliverables:**
- Compiling Zig project with dependencies
- `zig build run` executes successfully
- README updated with build instructions

### 1.2 Convert 3 Commands to YAML
**MVP Command Selection:**
1. `sanity-check` - Simplest (no GitHub calls)
2. `gh-work` - Core workflow (GitHub integration)
3. `create-issue` - Multi-step workflow

**Tasks:**
- [ ] Read existing `.md` files
- [ ] Extract parameters from `{{argN}}` placeholders
- [ ] Define validation rules for each parameter
- [ ] Write YAML files to `commands/`
- [ ] Keep original `.md` in `docs/commands/` for reference

**Deliverables:**
- 3 valid YAML command files
- Schema validation (manual review)

### 1.3 Core MCP Protocol Implementation
**Tasks:**
- [ ] Implement `src/mcp.zig` using ZigJR
- [ ] Register 4 JSON-RPC handlers:
  - `initialize`
  - `notifications/initialized`
  - `tools/list`
  - `tools/call`
- [ ] Implement stdio event loop in `main.zig`
- [ ] Handle JSON-RPC message parsing/serialization
- [ ] Return proper MCP responses

**Deliverables:**
- MCP server responds to `initialize` handshake
- `tools/list` returns 3 MVP commands
- `tools/call` returns prompt text (no execution yet)

### 1.4 Parameter Validation System
**Tasks:**
- [ ] Implement `src/validator.zig`
- [ ] Type validation (string, integer, boolean)
- [ ] Custom pattern validators:
  - Repository format (`owner/name`)
  - Issue numbers (positive integers)
  - Branch names (alphanumeric + `/_-`)
- [ ] Range validation (min/max)
- [ ] Return structured error responses

**Deliverables:**
- Validator blocks invalid input
- Returns detailed error messages
- JSON-RPC error envelope (-32602)

### 1.5 GitHub CLI Integration
**Tasks:**
- [ ] Implement `src/github.zig`
- [ ] Execute `gh` CLI via `std.process.Child.exec`
- [ ] Capture stdout/stderr
- [ ] Parse JSON responses
- [ ] Handle command failures
- [ ] Propagate errors to caller

**Deliverables:**
- Can execute `gh issue view 42 --json ...`
- Parse JSON response successfully
- Return structured data to command handler

**Milestone 1 Success Criteria:**
- ✅ Zig project compiles and runs
- ✅ Responds to MCP initialize handshake
- ✅ Lists 3 commands via `tools/list`
- ✅ Validates parameters correctly
- ✅ Can execute `gh` CLI and parse output

---

## Milestone 2: MVP Implementation (3 Commands)

**Goal:** Full end-to-end execution of 3 commands

### 2.1 Command Loader
**Tasks:**
- [ ] Scan `commands/` directory for `.yaml` files
- [ ] Parse each file using zig-yaml
- [ ] Build command registry (HashMap: name → Command)
- [ ] Validate YAML schema on load
- [ ] Report errors for malformed files

**Deliverables:**
- Commands loaded at startup
- Registry populated
- Schema validation enforced

### 2.2 Implement Command: `sanity-check`
**Tasks:**
- [ ] Implement handler in `src/commands/sanity_check.zig`
- [ ] Check project structure exists
- [ ] Validate git repository
- [ ] Return formatted prompt
- [ ] Test with Claude Code

**Deliverables:**
- `sanity-check` command works end-to-end
- No GitHub integration (simplest test)

### 2.3 Implement Command: `gh-work`
**Tasks:**
- [ ] Implement handler in `src/commands/gh_work.zig`
- [ ] Call `github.execGh()` to fetch issue
- [ ] Parse JSON response
- [ ] Substitute `{{issue_number}}` in prompt
- [ ] Return formatted prompt with context
- [ ] Test with Claude Code

**Deliverables:**
- `gh-work` fetches real GitHub issues
- Prompt includes issue title, body, labels

### 2.4 Implement Command: `create-issue`
**Tasks:**
- [ ] Implement handler in `src/commands/create_issue.zig`
- [ ] Multi-step workflow:
  1. Prompt for issue details (return prompt)
  2. Investigate codebase (use glob/grep patterns)
  3. Build issue body (template)
  4. Create via `gh issue create`
- [ ] Handle approval workflow
- [ ] Test with Claude Code

**Deliverables:**
- `create-issue` creates real GitHub issues
- End-to-end workflow validated

### 2.5 End-to-End Testing
**Tasks:**
- [ ] Configure MCP server in Claude Desktop settings
- [ ] Test each command manually:
  - Valid parameters
  - Invalid parameters (validation errors)
  - Missing required parameters
  - GitHub CLI errors
- [ ] Document test results

**Deliverables:**
- All 3 commands work in Claude Code
- Error handling verified
- Performance acceptable (< 100ms startup)

**Milestone 2 Success Criteria:**
- ✅ `sanity-check` command executes successfully
- ✅ `gh-work` fetches GitHub issues
- ✅ `create-issue` creates issues via gh CLI
- ✅ Parameter validation blocks invalid input
- ✅ Tested end-to-end with Claude Code client

---

## Milestone 3: Full Command Port (22 Remaining Commands)

**Goal:** Convert all 25 RAPID commands to YAML and implement handlers

### 3.1 Batch Convert Remaining Commands
**Command Categories:**

**GitHub Commands (8 more):**
- [ ] gh-finish
- [ ] gh-update-issue
- [ ] gh-review-issue
- [ ] gh-validate-issue
- [ ] gh-next-issue
- [ ] gh-create-milestone
- [ ] create-pr
- [ ] review-pr

**Documentation Commands (3):**
- [ ] doc-project
- [ ] create-doc
- [ ] shard-doc

**Workflow/Analysis Commands (8):**
- [ ] brainstorm
- [ ] elicit
- [ ] test-design
- [ ] qa-gate
- [ ] qa-apply-fixes
- [ ] nfr-assess
- [ ] risk-assess
- [ ] correct-course

**Utility Commands (3):**
- [ ] research-prompt
- [ ] gen-prompt
- [ ] trace-requirements

### 3.2 Category-Based Validation
**Tasks:**
- [ ] Add category-specific validation rules
- [ ] GitHub category: verify `gh` authenticated
- [ ] Documentation category: verify `docs/` directory exists
- [ ] Add metadata fields for requirements (git, gh CLI, etc.)

### 3.3 Comprehensive Error Handling
**Tasks:**
- [ ] Structured error responses (JSON-RPC)
- [ ] File-based logging (`/tmp/rapid-mcp.log`)
- [ ] Detailed validation error messages
- [ ] GitHub CLI error propagation
- [ ] Timeout handling for long-running commands

### 3.4 Testing & Documentation
**Tasks:**
- [ ] Unit tests for each command handler
- [ ] Integration tests with real GitHub API
- [ ] Performance benchmarks (startup time, command latency)
- [ ] Document adding new commands (template YAML)
- [ ] Update README with full command list

**Milestone 3 Success Criteria:**
- ✅ All 25 commands converted to YAML
- ✅ All commands execute successfully
- ✅ Comprehensive error handling
- ✅ Tests pass
- ✅ Documentation complete

---

## Milestone 4: HTTP Bridge (Optional - Scope Clarification)

**Status:** OPTIONAL - Not required for core MCP server functionality

**Goal:** Support non-MCP clients (VS Code tasks, legacy tools, manual testing)

**Recommendation:** Skip for MVP. Add only if non-MCP client support becomes a hard requirement.

**Why Optional:**
- MCP clients (Claude Code, Copilot, etc.) are the primary use case
- Adds http.zig dependency (contradicts "zero dependencies" goal)
- VS Code can use MCP protocol directly (no HTTP bridge needed)
- Increases complexity and attack surface

**If Implemented:**

### 4.1 HTTP Server Implementation
**Tasks:**
- [ ] Add `http.zig` (httpz library) dependency
- [ ] Implement HTTP server in `src/http_bridge.zig`
- [ ] Single endpoint: `POST /run`
- [ ] Request format: `{"command": "gh-work", "args": {...}}`
- [ ] Bridge HTTP → MCP command handlers
- [ ] Return JSON response
- [ ] Listen on `localhost:5001` (configurable)

### 4.2 Configuration System
**Tasks:**
- [ ] Environment variable support:
  - `RAPID_MCP_PORT` (default: 5001)
  - `RAPID_MCP_COMMANDS_DIR` (default: ./commands)
  - `RAPID_MCP_LOG_FILE` (default: /tmp/rapid-mcp.log)
- [ ] Config file support (optional): `config.yaml`

### 4.3 VS Code Integration
**Tasks:**
- [ ] Create `.vscode/tasks.json` examples
- [ ] Document VS Code task configuration
- [ ] Test with Copilot integration

**Milestone 4 Success Criteria:**
- ✅ HTTP bridge running on localhost:5001
- ✅ VS Code tasks can call commands
- ✅ Non-MCP clients supported
- ✅ Configuration via environment variables

---

## Milestone 5: RapidOS Integration Prep

**Goal:** Prepare for RapidOS system-level integration

### 5.1 System Integration Metadata
**Tasks:**
- [ ] Extend YAML schema with `os_integration:` section
- [ ] Define system permission requirements
- [ ] Add sandbox profile definitions
- [ ] Document security boundaries

**Example:**
```yaml
metadata:
  os_integration:
    requires_git: true
    requires_gh_cli: true
    system_permissions:
      - filesystem:read
      - network:github
    sandbox_profile: "github-workflow"
```

### 5.2 Single Binary Distribution
**Tasks:**
- [ ] Static compilation (no libc dependency)
- [ ] Cross-compile for:
  - Linux x86_64
  - Linux ARM64
  - macOS x86_64 (Intel)
  - macOS ARM64 (Apple Silicon)
  - Windows x86_64 (optional)
- [ ] Package as system service
- [ ] Create systemd unit file (Linux)
- [ ] Create launchd plist (macOS)

### 5.3 RapidOS Packaging
**Tasks:**
- [ ] Create RPM/DEB packages
- [ ] Install to `/usr/local/bin/rapid-mcp-server`
- [ ] System service configuration
- [ ] Integration with RapidOS AI orchestration layer

**Milestone 5 Success Criteria:**
- ✅ Static binary builds successfully
- ✅ Cross-platform compatibility verified
- ✅ System service configuration complete
- ✅ Ready for RapidOS integration

---

## Technology Stack

### Core Dependencies

**Build System:**
- Zig 0.14.0+ (or latest stable)

**Libraries:**
1. **ZigJR** - JSON-RPC 2.0 library (foundation only)
   - Repository: https://github.com/williamw520/zigjr
   - License: MIT
   - Features: Type-safe RPC, stdio transport, JSON-RPC 2.0 compliance
   - **Important:** Provides JSON-RPC infrastructure only. MCP protocol layer must be implemented manually.

2. **zig-yaml** - YAML parsing
   - Repository: https://github.com/kubkon/zig-yaml
   - License: MIT
   - Features: YAML 1.2 compatible, pure Zig
   - **Note:** Maintainer describes as "work-in-progress" - performance characteristics unverified

3. **http.zig (httpz)** - HTTP server (Milestone 4)
   - Repository: https://github.com/karlseguin/http.zig
   - License: MIT
   - Features: Pure Zig, ~140K req/s performance

**External Tools:**
- GitHub CLI (`gh`) - Required for GitHub operations
- git - Required for repository context

---

## Command Inventory (25 Total)

### GitHub Operations (11 commands)
1. `sanity-check` - Project health check
2. `gh-work` - Work on GitHub issue (investigate → plan → execute)
3. `gh-finish` - Complete and close issue
4. `gh-update-issue` - Update existing issue
5. `gh-review-issue` - Review and update issue
6. `gh-validate-issue` - Validate issue readiness
7. `gh-next-issue` - Fetch next issue to work on
8. `create-issue` - Create new GitHub issue with context
9. `gh-create-milestone` - Create GitHub milestone
10. `create-pr` - Create pull request
11. `review-pr` - Review pull request

### Documentation Commands (3)
12. `doc-project` - Generate project documentation
13. `create-doc` - Create new documentation file
14. `shard-doc` - Split large docs into smaller files

### Workflow/Analysis Commands (8)
15. `brainstorm` - Facilitate brainstorming session
16. `elicit` - Elicit requirements from stakeholders
17. `test-design` - Design test strategy
18. `qa-gate` - Quality assurance checkpoint
19. `qa-apply-fixes` - Apply QA fixes
20. `nfr-assess` - Non-functional requirements assessment
21. `risk-assess` - Risk assessment
22. `correct-course` - Course correction guidance

### Utility Commands (3)
23. `research-prompt` - Generate research prompt
24. `gen-prompt` - Generate specialized prompt
25. `trace-requirements` - Trace requirements to implementation

---

## Success Metrics

### Performance Targets

**Note:** These are aspirational targets, not verified benchmarks. Actual performance will be measured during implementation.

- **Startup time:** Target < 50ms (cold start) - *unverified until benchmarking complete*
- **Command latency:** Target < 100ms (YAML load + validation) - *depends on zig-yaml performance*
- **GitHub CLI overhead:** ~500ms+ (network dependent) - *will dominate total latency in practice*
- **Memory footprint:** Target < 10MB (resident) - *unverified, depends on command registry size*
- **Binary size:** Target < 5MB (static binary) - *depends on final dependency tree*

**Reality check:** GitHub CLI network calls will be the dominant latency factor (500ms+), making server startup time optimizations less impactful for user-perceived performance.

### Quality Targets
- **Test coverage:** > 80% (unit + integration)
- **Zero crashes:** No panics in production
- **Validation coverage:** 100% (all parameters validated)
- **Error handling:** 100% (all errors propagated correctly)

### User Experience Targets
- **Clear error messages:** Actionable feedback for validation failures
- **Fast feedback:** Commands respond within 1 second
- **Reliable:** No silent failures
- **Compatible:** Works with all MCP clients

---

## Testing Strategy

### Unit Tests
- Command loader (YAML parsing)
- Parameter validator (all validation rules)
- GitHub CLI wrapper (mocked responses)
- MCP protocol handlers (request/response)

### Integration Tests
- End-to-end command execution
- Real GitHub API calls (test repository)
- MCP client simulation
- Error scenarios (network failures, auth failures)

### Performance Tests
- Startup time benchmarks
- Command latency measurements
- Memory profiling
- Concurrency testing (multiple clients)

### Compatibility Tests
- Claude Code CLI
- GitHub Copilot
- Codex CLI
- VS Code tasks
- Generic MCP clients

---

## Implementation Reality Check

### This Is Pioneering Work

**Fact:** No mature Zig MCP server implementations exist as reference.
- lsp-mcp-server is an LSP↔MCP bridge, not a pure MCP server
- zig-mcp is 88.6% TypeScript, not a Zig implementation guide
- ZigJR provides JSON-RPC only, not MCP protocol

**Implication:** You'll be building the MCP layer from scratch on top of ZigJR.

### Dependency Contradictions

**Goal:** "Zero dependencies, single static binary"

**Reality:** Will likely need:
1. zig-yaml (required for YAML parsing) ✅ Acceptable
2. Template engine (for conditional prompt logic) ⚠️ Contradicts goal
3. Regex library (for robust validation) ⚠️ Contradicts goal
4. http.zig (if HTTP bridge implemented) ⚠️ Contradicts goal

**Decision Point:** Accept dependencies or simplify feature scope?

### Unverified Assumptions

**Performance Claims:**
- "< 50ms startup" - Unverified with YAML parsing overhead
- "< 100ms command latency" - Depends on zig-yaml performance (marked "work in progress")
- Network-bound `gh` CLI calls (500ms+) will dominate actual latency anyway

**YAML Schema:**
- Prototype revealed need for template engine
- 25 commands may have varying complexity levels
- Some commands may not fit schema cleanly

**Timeline:**
- Original: "6-8 weeks"
- Realistic: **8-12 weeks** accounting for:
  - Learning Zig ecosystem edge cases
  - Building MCP protocol layer from scratch
  - Discovering and resolving YAML conversion challenges
  - Testing with real AI clients

### Success Criteria Clarification

**MVP Definition:**
- 3 working commands (sanity-check, gh-work, create-issue)
- MCP protocol compliance (verified with Claude Code)
- Parameter validation working
- GitHub CLI integration working
- **Accept:** Simple string substitution, no template engine
- **Accept:** Hand-coded validation, no regex library
- **Skip:** HTTP bridge (Milestone 4 truly optional)

**Post-MVP Features:**
- Remaining 22 commands
- Template engine for complex workflows
- Regex library for robust validation
- HTTP bridge (if needed)

### Contingency Plan

**If Zig proves too difficult:**
- **Fallback Option 1:** Go implementation (better stdlib, mature ecosystem)
- **Fallback Option 2:** Rust implementation (strong typing, good FFI)
- **Fallback Option 3:** Node.js (works, but abandons performance goals)

**Decision point:** After Milestone 1 completion (Foundation)
- If ZigJR integration is smooth → Continue
- If hitting too many roadblocks → Pivot to Go/Rust

---

## Risk Management

### Technical Risks

**Risk 1:** zig-yaml library has limitations
- **Likelihood:** Medium
- **Impact:** High
- **Mitigation:** Test with complex YAML early; fallback to JSON format if needed
- **Contingency:** Use `std.json` with custom schema validation

**Risk 2:** ZigJR doesn't handle all MCP edge cases
- **Likelihood:** Low
- **Impact:** Medium
- **Mitigation:** Reference `examples/mcp_hello.zig` closely; contribute fixes upstream
- **Contingency:** Fork and patch if necessary

**Risk 3:** GitHub CLI output format changes
- **Likelihood:** Low
- **Impact:** Medium
- **Mitigation:** Pin to specific `gh` version; test against known schemas
- **Contingency:** Add version detection and adaptation layer

**Risk 4:** Performance issues with 25 commands loaded
- **Likelihood:** Low
- **Impact:** Low
- **Mitigation:** Benchmark early; lazy-load commands on first use if needed
- **Contingency:** Optimize YAML parsing or cache parsed commands

### Community/Adoption Risks

**Risk 5:** Zig community skepticism
- **Likelihood:** Medium
- **Impact:** Low
- **Mitigation:** Build in public, share progress, contribute to ecosystem
- **Contingency:** Focus on technical merit over community approval

**Risk 6:** RapidOS positioning unclear
- **Likelihood:** Medium
- **Impact:** Medium
- **Mitigation:** Clear messaging: "AI-first distro built in Zig"
- **Contingency:** Standalone tool with optional RapidOS integration

---

## Timeline & Milestones

### Milestone 1: Foundation (Weeks 1-2)
**Duration:** 1-2 weeks
**Deliverables:**
- Zig project setup with dependencies
- 3 commands converted to YAML
- Core MCP protocol implementation
- Parameter validation system
- GitHub CLI integration

### Milestone 2: MVP (Week 3)
**Duration:** 1 week
**Deliverables:**
- Command loader
- 3 commands fully implemented
- End-to-end testing with Claude Code

### Milestone 3: Full Port (Weeks 4-6)
**Duration:** 2-3 weeks
**Deliverables:**
- All 25 commands converted and implemented
- Comprehensive error handling
- Full test suite
- Documentation

### Milestone 4: HTTP Bridge (Week 7)
**Duration:** 1 week
**Deliverables:**
- HTTP server implementation
- VS Code integration examples
- Configuration system

### Milestone 5: RapidOS Prep (Week 8)
**Duration:** 1 week
**Deliverables:**
- System integration metadata
- Cross-platform binaries
- System service configuration

**Total Timeline:** 8-12 weeks (revised from original 6-8 week estimate)

**Timeline Adjustment Reasoning:**
- Building MCP protocol layer from scratch (no reference implementation)
- YAML conversion complexity discovered (template engine requirements)
- Learning curve for Zig ecosystem edge cases
- Integration testing with multiple MCP clients

---

## Open Questions

1. **Command versioning:** How to handle command schema evolution?
   - **Proposed:** Version field in YAML; support multiple versions simultaneously

2. **Hot reload:** Should server reload YAML files on change?
   - **Proposed:** Nice for dev, skip for MVP; add in Milestone 3

3. **Multi-repo support:** Can commands work across different git repos?
   - **Proposed:** Already supported via `-R owner/repo` flag in gh CLI

4. **Telemetry:** Should we add usage metrics for RapidOS?
   - **Proposed:** Milestone 5 consideration; opt-in only

5. **Backward compatibility:** How to handle breaking changes to commands?
   - **Proposed:** Semantic versioning; deprecation warnings

---

## References

### MCP Protocol
- Official docs: https://modelcontextprotocol.io
- Specification: https://github.com/modelcontextprotocol/specification
- JSON-RPC 2.0: https://www.jsonrpc.org/specification

### Zig Libraries
- ZigJR: https://github.com/williamw520/zigjr
- zig-yaml: https://github.com/kubkon/zig-yaml
- http.zig: https://github.com/karlseguin/http.zig

### Related Projects

**Important:** No pure Zig MCP server implementations exist as direct references. This project will be pioneering Zig MCP server development.

- **lsp-mcp-server** (LSP↔MCP bridge in Zig): https://github.com/nzrsky/lsp-mcp-server
  - Written in Zig, but bridges LSP to MCP rather than being a pure MCP server
  - Useful for understanding Zig architecture patterns, not MCP implementation details

- **zig-mcp** (Zig documentation MCP server): https://github.com/zig-wasm/zig-mcp
  - **Primarily TypeScript (88.6%)**, not Zig (only 2.2% Zig code)
  - Useful for understanding MCP server patterns, but not Zig implementation

### GitHub CLI
- gh CLI docs: https://cli.github.com/manual/
- gh CLI reference: https://cli.github.com/manual/gh

---

## Changelog

**Version 1.1 (2025-10-29)**
- **Critical corrections:** Clarified ZigJR provides JSON-RPC only (MCP layer must be built manually)
- **Reference projects:** Corrected descriptions (lsp-mcp-server is bridge, zig-mcp is TypeScript)
- **Performance claims:** Converted absolute claims to targets with disclaimers
- **Security:** Added shell injection prevention strategy
- **Validation:** Reconciled "no regex" contradiction, documented limitations
- **YAML prototype:** Converted sanity-check.md to YAML, discovered template engine requirement
- **Scope clarification:** HTTP bridge marked truly optional
- **Reality check:** Added "Implementation Reality Check" section
- **Timeline:** Revised to 8-12 weeks (from 6-8 weeks)
- **Dependencies:** Acknowledged contradictions with "zero dependencies" goal

**Version 1.0 (2025-10-28)**
- Initial PRD based on brainstorming session
- Defined 5-phase implementation plan
- Specified YAML command schema
- Outlined MCP protocol requirements
- Identified technology stack (ZigJR, zig-yaml, http.zig)
- Estimated 6-8 week timeline

---

## Approval & Next Steps

**Status:** ✅ Ready for Implementation

**Next Actions:**
1. Review and approve PRD
2. Initialize Zig project in this repository
3. Set up `build.zig` with dependencies
4. Begin Milestone 1: Foundation

**Prepared by:** John Junkins (@macjunkins)
**Date:** 2025-10-28
**Location:** /Users/johnjunkins/GitHub/rapid-mcp-server/prd.md
