# convenient-claude

A Rust CLI package manager for Claude Code resources — skills, agents, commands, hooks, and rules.

## Overview

`convenient-claude` (`cc`) is a single native binary that manages all Claude Code resource types through their full lifecycle: discover, install, configure, validate, update, and remove. It also provides session management with token usage tracking.

### Why this exists

Existing tools in the Claude Code ecosystem each handle a narrow slice of resource management:

| Tool | Scope | Gap |
|------|-------|-----|
| claude-cmd | Commands only (registry-based) | No skills, hooks, agents, rules |
| caude-skill-manager | Skills only | No other resource types |
| ccexp | Read-only TUI explorer | Cannot install or manage |
| claude-code-tool-manager | MCP/skills/agents | Desktop GUI only, no CLI |

**convenient-claude** fills the gap: a unified CLI that handles all 5 resource types with full lifecycle management, plus session tracking and stats analytics.

## Features

- **Resource Discovery**: Scan and list resources from multiple origins (external libraries, user folder, project folder)
- **Resource Installation**: Install skills, commands, agents, and rules from external sources
- **Conflict Resolution**: Precedence-based merging when resources exist in multiple origins
- **Session Management**: Track active skills/agents with token and resource usage stats
- **Validation**: Validate all project resources with auto-fix capabilities
- **Configuration Merging**: Three-way merge of user, project, and session settings
- **Interactive TUI**: Full-screen terminal dashboard for browsing and inspecting resources
- **Shell Completions**: Generate completions for bash, zsh, fish, powershell

## Installation

### From source

```bash
git clone https://github.com/your-username/convenient-claude.git
cd convenient-claude
cargo build --release
```

The binary will be at `target/release/cc`. Add it to your PATH.

### From Cargo (future)

```bash
cargo install convenient-claude
```

## Usage

### Initialize a project

```bash
cd my-project
cc init
```

Creates `.claude/` directory structure with:
- `skills/`, `commands/`, `agents/`, `rules/` directories
- `settings.json` with empty permissions

### List resources

```bash
cc list skills               # List all skills
cc list commands             # List all commands
cc list agents               # List all agents
cc list all                  # List everything
cc list skills --format json # JSON output
cc list skills --format plain # One per line
```

### Install resources

```bash
cc add skill react-expert --from extern/claude-skills
cc add command commit --from extern/everything-claude-code
cc add hook PreToolUse "lint-check.sh" --matcher "Bash"
```

### Remove resources

```bash
cc remove skill react-expert
cc remove command commit
cc remove hook PreToolUse "lint-check.sh"
```

### Show resource details

```bash
cc show skill react-expert
cc show command commit
```

### Validate project

```bash
cc validate                  # Check all resources
cc validate --fix            # Auto-fix warnings
```

### Diagnose issues

```bash
cc doctor                    # Check configuration health
```

### Interactive TUI dashboard

```bash
cc tui                      # Launch full-screen TUI
```

Keybindings inside the TUI:
- `↑`/`↓` or `j`/`k` — Navigate resources
- `Tab`/`Shift+Tab` — Switch resource type (Skills, Commands, Agents, Rules)
- `Enter` — Toggle detail panel for selected resource
- `r` — Refresh resource list
- `q` or `Esc` — Quit

### Session management

```bash
cc session start --mode interactive --skills commit,review
cc session status            # Show active session info
cc session stats             # Token and resource usage
cc session stop              # End session and save stats
```

### Stats analytics

```bash
cc stats session             # Current session stats
cc stats history --last 5    # Recent sessions
cc stats resources           # Per-resource breakdown
```

### Configuration

```bash
cc config show               # Merged settings
cc config get permissions.allow
cc config set permissions.allow "Bash(cargo build:*)" --scope project
```

## Architecture

Three-layer architecture with strict one-directional dependencies:

```
┌─────────────────────────────────────────────────────────┐
│                    CLI Layer (cc-cli)                    │
│  clap derive • command dispatch • output formatting     │
├─────────────────────────────────────────────────────────┤
│                  Service Layer (cc-core)                 │
│  resource resolution • session management • validation  │
├─────────────────────────────────────────────────────────┤
│                   Data Layer (cc-schema)                 │
│  schema definitions • frontmatter parsing • file I/O    │
└─────────────────────────────────────────────────────────┘
```

### Resource resolution precedence

```
Session overrides   ← highest precedence
Project .claude/
User ~/.claude/
External extern/    ← lowest precedence (read-only)
```

### Crate structure

| Crate | Files | Purpose |
|-------|-------|---------|
| `cc-cli` | 12 | Argument parsing, command handlers, output formatting |
| `cc-core` | 23 | Business logic, resource discovery, session tracking |
| `cc-schema` | 12 | Data structures, YAML frontmatter parsing, I/O utilities |

## Development

### Requirements

- Rust 1.77+ (edition 2021)
- Cargo

### Commands

```bash
cargo build          # Build all crates
cargo test --workspace # Run 34 tests
cargo fmt            # Format code
cargo clippy         # Lint
cargo run -- help    # Run CLI with help
```

### Adding a new command

1. Add clap subcommand in `crates/cc-cli/src/args.rs`
2. Create handler in `crates/cc-cli/src/commands/`
3. Add service logic in `crates/cc-core/` if needed
4. Add schema support in `crates/cc-schema/` if needed

## Resource Types

| Type | Format | Location |
|------|--------|----------|
| Skill | Markdown + YAML frontmatter | `.claude/skills/*.md` |
| Command | Markdown + YAML frontmatter | `.claude/commands/*.md` |
| Agent | Markdown + YAML frontmatter | `.claude/agents/*.md` |
| Hook | JSON (in settings.json) | `.claude/settings.json` |
| Rule | Plain Markdown | `.claude/rules/*.md` |

## Test Coverage

- **23 tests** in `cc-schema`: frontmatter parsing, I/O, schema validation
- **11 tests** in `cc-core`: path resolution, precedence, config merge, token estimation

## License

MIT

## Contributing

1. Fork the repository
2. Create a feature branch
3. Run `cargo fmt && cargo clippy && cargo test`
4. Submit a pull request