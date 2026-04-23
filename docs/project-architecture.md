# Project Architecture

## 1. Overview

**convenient-claude** (`cc`) is a Rust CLI that acts as a **package manager for Claude Code resources** — skills, agents, commands, hooks, and rules. It discovers, installs, validates, and manages these resources across three origins:

| Origin | Location | Scope | Editable |
|--------|----------|-------|----------|
| External libraries | `extern/` (git submodules) | Shared community resources | No (read-only, synced via git) |
| Local user folder | `~/.claude/` | User-wide defaults | Yes |
| Project folder | `.claude/` within a project | Project-specific configuration | Yes |

### Why this project exists

Existing tools in the Claude Code ecosystem each handle a narrow slice of resource management:

| Tool | Language | Scope | Gap |
|------|----------|-------|-----|
| **claude-cmd** (kiliczsh) | TypeScript | Commands only (184+ registry) | No skills, hooks, agents, or rules |
| **caude-skill-manager** (majiayu000) | Unknown | Skills only, registry-based | No other resource types |
| **ccexp** (nyatinte) | TypeScript | Read-only TUI explorer | Cannot install or manage |
| **claude-code-tool-manager** (tylergraydev) | Rust/Tauri GUI | MCP servers, skills, sub-agents | Desktop GUI only, no CLI |
| **claude-config-composer** (Matt-Dionis) | TypeScript | One-shot config generation | No ongoing management |
| **CCManager** (kbwo) | TypeScript | Session management only | No resource management |

**convenient-claude** fills the gap: a single native binary that manages **all** Claude Code resource types through their full lifecycle (discover, install, configure, validate, update, remove), with session management on top.

### Architecture Layers

```
┌──────────────────────────────────────────────────────────┐
│                       CLI Layer                           │
│                    (crates/cc-cli)                        │
│  clap derive • command dispatch • output formatting       │
│  interactive prompts • shell completions                  │
├──────────────────────────────────────────────────────────┤
│                     Service Layer                         │
│                    (crates/cc-core)                       │
│  resource resolution • session management                 │
│  conflict resolution • validation • sync                  │
├──────────────────────────────────────────────────────────┤
│                      Data Layer                           │
│                   (crates/cc-schema)                      │
│  schema definitions • frontmatter parsing                 │
│  JSON/TOML/YAML serialization • file system I/O           │
└──────────────────────────────────────────────────────────┘
```

**Dependency graph:** `cc-cli → cc-core → cc-schema` — strictly one-directional, no circular dependencies.

---

## 2. CLI Command Tree

The binary is named `cc` (convenient-claude). Full command hierarchy:

```
cc
├── init                          # Initialize .claude/ in a project
├── list                          # List resources
│   ├── list skills [FILTER]      # List skills
│   ├── list commands [FILTER]    # List commands
│   ├── list agents [FILTER]      # List agents
│   ├── list hooks                # List hooks
│   ├── list rules [FILTER]       # List rules
│   └── list all                  # List everything, grouped by type
├── add                           # Install a resource
│   ├── add skill <NAME>          # Install a skill
│   │   ├── --from <SOURCE>       #   Source: extern/<lib>, url, file path
│   │   ├── --to <SCOPE>          #   Target: project (default), user
│   │   └── --force               #   Overwrite if exists
│   ├── add command <NAME>        # Install a command
│   ├── add agent <NAME>          # Install an agent
│   ├── add hook <EVENT> <CMD>    # Register a hook
│   └── add rule <NAME>           # Install a rule
├── remove                        # Uninstall a resource
│   ├── remove skill <NAME>
│   ├── remove command <NAME>
│   ├── remove agent <NAME>
│   ├── remove hook <EVENT> <CMD>
│   └── remove rule <NAME>
├── show <TYPE> <NAME>            # Display resource details + origin
├── validate                      # Validate all project resources
│   └── --fix                     # Auto-fix what's fixable
├── sync                          # Sync extern/ submodules
│   └── --dry-run                 # Preview changes without applying
├── session                       # Session management
│   ├── session start             # Start a managed session
│   │   ├── --mode <MODE>         #   conversation | loop | interactive
│   │   ├── --skills <LIST>       #   Activate specific skills
│   │   ├── --agents <LIST>       #   Activate specific agents
│   │   └── --config <FILE>       #   Load session config from file
│   ├── session stop              # Stop active session
│   ├── session status            # Show active session info
│   └── session stats             # Show resource and token usage stats
├── config                        # View/edit merged configuration
│   ├── config show               # Print effective (merged) settings
│   ├── config get <KEY>          # Get a specific config value
│   ├── config set <KEY> <VALUE>  # Set a config value
│   │   └── --scope <SCOPE>       #   project | user
│   └── config diff               # Show overrides vs. user defaults
├── stats                         # Resource and token usage analytics
│   ├── stats session             # Current session stats (live)
│   ├── stats history             # Historical session stats
│   │   └── --last <N>            #   Last N sessions (default: 10)
│   └── stats resources           # Per-resource usage breakdown
├── doctor                        # Diagnose setup issues
└── completions                   # Generate shell completions
    └── <SHELL>                   # bash | zsh | fish | powershell
```

### Command flags (global)

| Flag | Short | Description |
|------|-------|-------------|
| `--verbose` | `-v` | Show detailed output |
| `--quiet` | `-q` | Suppress non-error output |
| `--project-dir <PATH>` | `-p` | Override project directory detection |
| `--no-color` | | Disable colored output |
| `--help` | `-h` | Show help |
| `--version` | `-V` | Show version |

### Output formats

Most `list` and `show` commands accept `--format`:

| Format | Flag | Description |
|--------|------|-------------|
| Table | `--format table` (default) | Human-readable columns |
| JSON | `--format json` | Machine-readable, pipeable |
| Plain | `--format plain` | One entry per line, for scripting |

---

## 3. Components

### 3.1 CLI Layer (`crates/cc-cli`)

**Responsibilities:** argument parsing, command dispatch, output formatting, interactive prompts.

#### CLI parsing — solution comparison

| Criteria | `clap` v4 (derive) | `clap` v4 (builder) | `lexopt` |
|----------|-------------------|---------------------|----------|
| Ergonomics | High — derive macros, type-safe | Medium — chained builders | Low — manual |
| Compile time | ~3-4s incremental | ~3-4s incremental | <1s |
| Binary size | ~500KB overhead | ~500KB overhead | <50KB |
| Features | Shell completions, help, env vars, subcommands | Same | Minimal |
| **Verdict** | **Use** — the subcommand tree is deep enough that derive macros save significant boilerplate. Compile time is a one-time cost. | | |

#### Terminal rendering — library comparison

The CLI targets PowerShell, bash, zsh, and fish. All output must render correctly across these terminals, including Windows Terminal + PowerShell where ANSI support was historically inconsistent.

**Styling (colors, bold, underline, etc.):**

| Library | Approach | Zero-alloc | Windows support | `const`-friendly | Notes |
|---------|----------|------------|-----------------|-------------------|-------|
| `owo-colors` | Trait extension | Yes | Via `supports-color` | Yes | Fastest, most ergonomic. Used by `clap` internally. |
| `colored` | Trait extension | No (allocs per call) | Yes (crossterm backend) | No | Simpler API but heavier. |
| `anstyle` | Value types | Yes | Yes | Yes | Minimal, no global mutex. Used by `clap` for `--help` colors. |
| `console` (console-rs) | `Style` builder | No | Yes | No | Bundles terminal detection, alignment, progress. Full-featured. |
| **Verdict** | **`owo-colors`** for pure styling (lightweight, zero-alloc, widest compatibility). **`console`** for its `Term` abstraction, text alignment (`pad_str`), and terminal capability detection. | | | | |

**Tables:**

| Library | Dynamic columns | Styling | Sorting | Markdown export | Notes |
|---------|----------------|---------|---------|-----------------|-------|
| `comfy-table` | Yes | Per-cell colors | Yes | No | Clean API, auto-width, good Windows support. |
| `tabled` | Yes | Per-cell | Yes | Yes | More features (inline mode, grid format). Heavier. |
| **Verdict** | **`comfy-table`** — simpler API, sufficient features, lighter dependency. | | | | |

**Markdown rendering in terminal:**

| Library | Purpose | Windows support | Notes |
|---------|---------|-----------------|-------|
| `termimad` | Render Markdown snippets in terminal | Yes (uses `crossterm`) | Skins for custom styling. Perfect for `cc show skill` previews, help text. |
| `comrak` | Full CommonMark parser | Yes | Overkill — it's for parsing, not terminal rendering. |
| **Verdict** | **`termimad`** — purpose-built for "display markdown in a terminal" with styled output, scrollable areas, and table support. | | |

**Progress bars and spinners:**

| Library | Purpose | Notes |
|---------|---------|-------|
| `indicatif` | Progress bars, spinners, multi-progress | De facto standard. Used for `cc sync`, batch install operations. |
| `console` (built-in) | Basic progress | Less feature-rich than `indicatif`, but avoids an extra dep. |
| **Verdict** | **`indicatif`** — richer features (multi-bar, ETA, bytes mode). | |

**Interactive prompts:**

| Library | Widgets | Fuzzy search | Validation | Notes |
|---------|---------|-------------|------------|-------|
| `dialoguer` | Select, MultiSelect, Confirm, Input | No | Yes | Simple, well-known. |
| `inquire` | Same + Password, Editor | Yes | Yes | More features, slightly heavier. |
| **Verdict** | **`inquire`** — fuzzy search is useful when browsing 60+ skills from extern/. | | |

**Full TUI framework (NOT recommended for this project):**

| Library | Use case | Why NOT to use here |
|---------|----------|-------------------|
| `ratatui` | Full-screen terminal apps (dashboards, editors) | Overkill — `cc` is a command-line tool, not an interactive TUI. Adds complexity for no benefit. |
| `cursive` | Same | Same reason, plus it brings its own backend (ncurses/pancurses). |

#### Key dependencies (revised)

```toml
[dependencies]
cc-core = { path = "../cc-core" }
clap = { version = "4", features = ["derive"] }
anyhow = "1"
# Terminal rendering
owo-colors = "4"          # Zero-alloc terminal colors
console = "0.15"          # Terminal abstraction, alignment, capability detection
comfy-table = "7"         # Table output
termimad = "0.31"         # Markdown rendering in terminal
indicatif = "0.17"        # Progress bars and spinners
inquire = "0.7"           # Interactive prompts with fuzzy search
serde_json = "1"          # JSON output format
```

#### Module structure

```
crates/cc-cli/src/
├── main.rs              # Entry point, top-level error handling
├── args.rs              # clap derive structs for all commands
├── commands/
│   ├── mod.rs           # Command dispatch trait
│   ├── init.rs          # cc init
│   ├── list.rs          # cc list <type>
│   ├── add.rs           # cc add <type> <name>
│   ├── remove.rs        # cc remove <type> <name>
│   ├── show.rs          # cc show <type> <name>
│   ├── validate.rs      # cc validate
│   ├── sync.rs          # cc sync
│   ├── session.rs       # cc session start/stop/status/stats
│   ├── stats.rs         # cc stats session/history/resources
│   ├── config_cmd.rs    # cc config show/get/set/diff
│   ├── doctor.rs        # cc doctor
│   ├── stats.rs         # cc stats session/history/resources
│   └── completions.rs   # cc completions <shell>
└── output.rs            # Formatting helpers (table, json, plain)
```

### 3.2 Service Layer (`crates/cc-core`)

**Responsibilities:** resource resolution, conflict merging, session management, validation, external sync.

#### Resource resolution

Resources are resolved from four layers with strict precedence (highest wins):

```
┌─────────────────────────────────┐
│ Session overrides               │  ← cc session start --skills X
├─────────────────────────────────┤
│ Project .claude/                │  ← .claude/skills/, .claude/commands/, etc.
├─────────────────────────────────┤
│ User ~/.claude/                 │  ← User-wide defaults
├─────────────────────────────────┤
│ External extern/                │  ← Community libraries (read-only)
└─────────────────────────────────┘
```

When a resource exists in multiple origins, the highest-precedence one wins. `cc list` shows all origins with an indicator of which is active.

#### Module structure

```
crates/cc-core/src/
├── lib.rs               # Public API re-exports
├── resource/
│   ├── mod.rs           # Resource trait, ResourceType enum
│   ├── discovery.rs     # Scan all origins for resources
│   ├── resolution.rs    # Merge across origins, apply precedence
│   └── install.rs       # Copy/link resource into target scope
├── skill.rs             # Skill-specific loading and validation
├── agent.rs             # Agent-specific loading and validation
├── command.rs           # Command-specific loading and validation
├── hook.rs              # Hook registration and management
├── rule.rs              # Rule loading and management
├── session/
│   ├── mod.rs           # Session lifecycle
│   ├── context.rs       # Active resource set for a session
│   ├── tracker.rs       # Token and resource usage tracking
│   └── modes.rs         # Conversation, loop, interactive modes
├── stats/
│   ├── mod.rs           # Stats aggregation and reporting
│   ├── token.rs         # Token counting and cost estimation
│   └── usage.rs         # Per-resource usage breakdown
├── config/
│   ├── mod.rs           # Config loading and merging
│   ├── settings.rs      # settings.json read/write
│   └── merge.rs         # Three-way merge (user + project + session)
├── sync/
│   ├── mod.rs           # External library sync
│   └── git.rs           # Git submodule operations
├── validate/
│   ├── mod.rs           # Validation orchestration
│   ├── schema.rs        # Schema validation per resource type
│   └── conflicts.rs     # Cross-resource conflict detection
├── origin.rs            # Origin enum, path resolution per origin
└── error.rs             # Domain error types
```

#### Key dependencies

```toml
[dependencies]
cc-schema = { path = "../cc-schema" }
walkdir = "2"            # Recursive directory traversal
glob = "0.3"             # Glob pattern matching for resource files
git2 = "0.19"            # Git operations (submodule sync)
# OR shell out to git CLI — simpler but requires git on PATH
regex = "1"
chrono = "0.4"           # Timestamps in session logs
tiktoken-rs = "0.6"     # Token counting for estimation
uuid = { version = "1", features = ["v4"] }  # Session IDs
tracing = "0.1"          # Structured logging
thiserror = "2"          # Derived error types
```

### 3.3 Data Layer (`crates/cc-schema`)

**Responsibilities:** schema definitions for all resource types, frontmatter parsing, serialization, file I/O.

#### Resource schemas (from real Claude Code formats)

**Skill** — Markdown with YAML frontmatter (from claude-skills and everything-claude-code):

```yaml
# .claude/skills/my-skill.md
---
name: my-skill
description: Brief description. Use when [triggering conditions].
license: MIT
metadata:
  author: Author Name
  version: "1.0.0"
  domain: backend | frontend | infrastructure | ...
  triggers: comma, separated, keywords
  role: specialist | expert | architect | engineer
  scope: implementation | review | design | testing | ...
  output-format: code | document | report | ...
  related-skills: other-skill-name
---

# Skill Name

[Skill instructions in Markdown]
```

```rust
// Rust struct
#[derive(Debug, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub metadata: SkillMetadata,
    // Body content (Markdown after frontmatter)
    #[serde(skip)]
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillMetadata {
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub triggers: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub output_format: Option<String>,
    #[serde(default)]
    pub related_skills: Option<String>,
}
```

**Command** — Markdown with YAML frontmatter:

```yaml
# .claude/commands/my-command.md
---
name: my-command
description: What this command does
allowed_tools: ["Bash", "Read", "Write"]
---

# /my-command

Instructions for the command...
```

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    #[serde(skip)]
    pub body: String,
}
```

**Agent** — Markdown with YAML frontmatter:

```yaml
# .claude/agents/my-agent.md
---
name: my-agent
description: Agent's purpose and capabilities
model: sonnet | opus | haiku
tools: ["Bash", "Read", "Write", "Grep", "Glob"]
---

# Agent instructions...
```

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Agent {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(skip)]
    pub body: String,
}
```

**Hook** — Stored in `settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [
      { "matcher": "Bash", "hooks": [{ "type": "command", "command": "my-script.sh" }] }
    ],
    "PostToolUse": [],
    "Notification": [],
    "Stop": []
  }
}
```

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct HookConfig {
    #[serde(default)]
    pub hooks: HashMap<HookEvent, Vec<HookMatcher>>,
}

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum HookEvent {
    PreToolUse,
    PostToolUse,
    Notification,
    Stop,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HookMatcher {
    #[serde(default)]
    pub matcher: Option<String>,
    pub hooks: Vec<HookEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HookEntry {
    #[serde(rename = "type")]
    pub hook_type: String,
    pub command: String,
}
```

**Settings** — `settings.json`:

```json
{
  "permissions": {
    "allow": ["Bash(cargo build)", "Bash(cargo test)"],
    "deny": []
  },
  "hooks": { ... }
}
```

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub permissions: Permissions,
    #[serde(default)]
    pub hooks: Option<HookConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Permissions {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}
```

**Rule** — Plain Markdown in `.claude/rules/`:

```rust
#[derive(Debug)]
pub struct Rule {
    pub name: String,      // Derived from filename
    pub body: String,      // Full Markdown content
    pub source_path: PathBuf,
}
```

**Session stats** — Tracked per-session in `.claude/session-stats.json`:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStats {
    pub session_id: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub stopped_at: Option<chrono::DateTime<chrono::Utc>>,
    pub mode: SessionMode,
    pub token_usage: TokenUsage,
    pub resource_usage: Vec<ResourceUsage>,
    pub tool_invocations: HashMap<String, u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    /// Estimated cost in USD (based on model pricing)
    pub estimated_cost: Option<f64>,
    /// Token breakdown by resource type
    pub by_resource: HashMap<ResourceType, TokenBreakdown>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenBreakdown {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub resource_type: ResourceType,
    pub name: String,
    pub origin: Origin,
    pub times_invoked: u32,
    pub tokens_consumed: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SessionMode {
    Conversation,
    Loop,
    Interactive,
}
```

#### Module structure

```
crates/cc-schema/src/
├── lib.rs               # Public re-exports
├── skill.rs             # Skill struct, frontmatter parsing
├── command.rs           # Command struct, frontmatter parsing
├── agent.rs             # Agent struct, frontmatter parsing
├── hook.rs              # HookEvent, HookMatcher, HookEntry
├── rule.rs              # Rule struct
├── settings.rs          # Settings, Permissions
├── resource_type.rs     # ResourceType enum (Skill, Command, Agent, Hook, Rule)
├── origin.rs            # Origin enum (External, User, Project)
├── frontmatter.rs       # YAML frontmatter extraction from Markdown
├── stats.rs             # SessionStats, TokenUsage, ResourceUsage structs
├── io.rs                # File read/write helpers
└── error.rs             # Schema-level error types
```

#### Key dependencies

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"       # YAML frontmatter parsing
pulldown-cmark = "0.11"  # Markdown parsing (for body extraction)
thiserror = "2"
```

---

## 4. Data Flow

### 4.1 Resource discovery

```
cc list skills --format table

 ┌─ CLI: parse args ─────────────────────────────────────────┐
 │  ListCmd { resource: Skill, filter: None, format: Table } │
 └───────────────────────────┬───────────────────────────────┘
                             │
 ┌─ Service: discover ───────▼───────────────────────────────┐
 │  for each origin (External, User, Project):               │
 │    scan <origin>/skills/ for *.md files                   │
 │    ┌─ Data: parse each file ──────────────────────────┐   │
 │    │  extract_yaml_frontmatter() → SkillMetadata       │   │
 │    │  extract_body() → Skill.body                      │   │
 │    │  assemble → Skill { name, description, ... }      │   │
 │    └───────────────────────────────────────────────────┘   │
 │  merge_by_name(skills_from_all_origins)                    │
 │  → Vec<ResourceEntry { skill, origin, active: bool }>     │
 └───────────────────────────┬───────────────────────────────┘
                             │
 ┌─ CLI: format ─────────────▼───────────────────────────────┐
 │  render_table(entries) →                                  │
 │  ┌──────────┬─────────────┬──────────┬────────┐           │
 │  │ Name     │ Description │ Origin   │ Active │           │
 │  ├──────────┼─────────────┼──────────┼────────┤           │
 │  │ commit   │ ...         │ project  │ ●      │           │
 │  │ review   │ ...         │ user     │ ●      │           │
 │  │ python   │ ...         │ external │        │           │
 │  └──────────┴─────────────┴──────────┴────────┘           │
 └───────────────────────────────────────────────────────────┘
```

### 4.2 Resource installation

```
cc add skill react-expert --from extern/claude-skills --to project

 ┌─ CLI ─────────────────────────────────────────────────────┐
 │  AddCmd { type: Skill, name: "react-expert",              │
 │           from: External("claude-skills"), to: Project }  │
 └───────────────────────────┬───────────────────────────────┘
                             │
 ┌─ Service ─────────────────▼───────────────────────────────┐
 │  1. locate("react-expert", origin=External)                │
 │     → extern/claude-skills/skills/react-expert.md          │
 │  2. load resource (via Data layer) → Skill                 │
 │  3. validate(skill) → check name, description, body        │
 │  4. check_conflicts("react-expert", target=Project)        │
 │     → no conflict (or prompt if --force)                   │
 │  5. install(skill, target=Project)                         │
 │     ┌─ Data ─────────────────────────────────────────┐     │
 │     │  write .claude/skills/react-expert.md           │     │
 │     │  (copy Markdown + frontmatter from source)      │     │
 │     └─────────────────────────────────────────────────┘     │
 │  6. return InstallResult { path, resource }                │
 └───────────────────────────┬───────────────────────────────┘
                             │
 ┌─ CLI ─────────────────────▼───────────────────────────────┐
 │  ✓ Installed skill "react-expert" → .claude/skills/        │
 │    Origin: extern/claude-skills                             │
 └───────────────────────────────────────────────────────────┘
```

### 4.3 Configuration merge

```
cc config show

User settings.json          Project settings.json        Merged result
┌──────────────────────┐    ┌──────────────────────┐    ┌──────────────────────┐
│ allow:               │    │ allow:               │    │ allow:               │
│   cargo build        │    │   cargo build:*      │    │   cargo build:*  ←P  │
│   cargo test         │    │   cargo clippy       │    │   cargo clippy   ←P  │
│ deny:                │    │ deny:                │    │   cargo test    ←U   │
│   Bash(rm *)         │    │                      │    │ deny:                │
└──────────────────────┘    └──────────────────────┘    │   Bash(rm *)   ←U   │
                                                        └──────────────────────┘
←P = project override   ←U = inherited from user
```

### 4.4 Session lifecycle with stats tracking

```
cc session start --mode interactive --skills commit,review

 ┌─ Service: create session context ──────────────────────────┐
 │  1. Load project resources                                  │
 │  2. Filter: only activate "commit" and "review" skills     │
 │  3. Load all project hooks                                  │
 │  4. Build effective settings (user + project + session)     │
 │  5. Initialize SessionStats { token_usage: 0, ... }        │
 │  6. Write session state to .claude/session.json             │
 │  7. Write empty stats to .claude/session-stats.json         │
 └────────────────────────────┬───────────────────────────────┘
                              │
 ┌─ Interactive loop ─────────▼───────────────────────────────┐
 │  ┌──────────┐   ┌──────────┐   ┌──────────┐               │
 │  │  Prompt   │──▶│ Execute  │──▶│ Confirm  │──▶ next turn  │
 │  └──────────┘   └────┬─────┘   └──────────┘               │
 │                      │                                     │
 │         ┌────────────▼──────────────┐                      │
 │         │  Stats tracker (per turn) │                      │
 │         │  • accumulate tokens      │                      │
 │         │  • log tool invocations   │                      │
 │         │  • attribute to resource  │                      │
 │         │  • flush to stats.json    │                      │
 │         └───────────────────────────┘                      │
 │                                                              │
 │  Mode behaviors:                                             │
 │  • conversation: single prompt → response → exit            │
 │  • loop:         prompt → execute → auto-next → repeat      │
 │  • interactive:  prompt → execute → confirm → repeat        │
 └──────────────────────────────────────────────────────────────┘
                              │
 ┌─ cc session stats ────────▼───────────────────────────────┐
 │  Read .claude/session-stats.json → format & display        │
 └────────────────────────────────────────────────────────────┘
                              │
 ┌─ cc session stop ─────────▼───────────────────────────────┐
 │  1. Finalize stats (set stopped_at)                        │
 │  2. Append session summary to .claude/stats-history.jsonl  │
 │  3. Clean up ephemeral session files                       │
 │  4. Print session summary with token usage                 │
 └─────────────────────────────────────────────────────────────┘
```

### 4.5 Token tracking data sources

Claude Code exposes token usage through its API responses and `--output-format json` mode. The stats tracker captures:

```
Claude Code API response / JSON output
  │
  ├── usage.input_tokens      ──▶  TokenUsage.input_tokens
  ├── usage.output_tokens     ──▶  TokenUsage.output_tokens
  ├── model                   ──▶  cost estimation lookup
  │
  ├── tool_use[].name         ──▶  ResourceUsage.times_invoked
  ├── skill invocations       ──▶  ResourceUsage.tokens_consumed
  └── per-turn accumulation   ──▶  flush to session-stats.json

Token counting approaches:
┌────────────────────┬─────────────────────┬──────────────────────┐
│ Source             │ Accuracy            │ Availability         │
├────────────────────┼─────────────────────┼──────────────────────┤
│ API response       │ Exact               │ Requires API access  │
│ --output-format    │ Exact               │ Per-invocation       │
│ tiktoken-rs count  │ Close approximation │ Always available     │
└────────────────────┴─────────────────────┴──────────────────────┘
```

---

## 5. Error Handling

### Layered error strategy

| Layer | Error type | Conversion |
|-------|-----------|------------|
| Data | `SchemaError` (thiserror enum) | I/O and parse errors with path context |
| Service | `CoreError` (thiserror enum) | Wraps `SchemaError`, adds domain context |
| CLI | `anyhow::Error` | Top-level catch, rich user-facing messages |

### Error types

```rust
// cc-schema
#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    #[error("file not found: {path}")]
    FileNotFound { path: PathBuf },
    #[error("invalid frontmatter in {path}: {reason}")]
    InvalidFrontmatter { path: PathBuf, reason: String },
    #[error("missing required field '{field}' in {path}")]
    MissingField { path: PathBuf, field: String },
    #[error("JSON parse error in {path}: {source}")]
    JsonError { path: PathBuf, source: serde_json::Error },
    #[error("I/O error: {source}")]
    Io { #[source] source: std::io::Error },
}

// cc-core
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("resource not found: {resource_type} '{name}'")]
    ResourceNotFound { resource_type: ResourceType, name: String },
    #[error("conflict: {resource_type} '{name}' exists in {existing_origin}, cannot install from {new_origin}")]
    Conflict { resource_type: ResourceType, name: String, existing_origin: Origin, new_origin: Origin },
    #[error("validation failed: {details}")]
    ValidationFailed { details: String },
    #[error("session already active")]
    SessionActive,
    #[error("no active session")]
    NoSession,
    #[error(transparent)]
    Schema(#[from] SchemaError),
}
```

### User-facing error output

```
Error: resource not found: skill 'react-expert'

  No skill named 'react-expert' was found in any origin.

  Searched:
    .claude/skills/          (not found)
    ~/.claude/skills/        (not found)
    extern/claude-skills/    (not found)

  Suggestions:
    • Run 'cc list skills' to see available skills
    • Run 'cc sync' to update external libraries
    • Check the skill name spelling
```

---

## 6. Testing

### Testing pyramid

```
               ┌───────────┐
               │   E2E /   │    cc-cli/tests/
               │   CLI     │    assert_cmd + predicates
              ┌┴───────────┴┐
              │ Integration  │   cc-core/tests/
              │              │   tempfile for real FS operations
             ┌┴──────────────┴┐
             │   Unit Tests    │  per-module #[cfg(test)]
             │                 │  no FS, no I/O
             └─────────────────┘
```

### Test matrix

| What | How | Where | Example |
|------|-----|-------|---------|
| Frontmatter parsing | Unit, inline | `cc-schema/src/skill.rs` | Parse YAML + body from markdown string |
| Settings merge | Unit, inline | `cc-core/src/config/merge.rs` | User allow + project allow = merged list |
| Resource discovery | Integration | `cc-core/tests/` | Create temp dir structure, scan, assert results |
| Conflict detection | Integration | `cc-core/tests/` | Same-named skill in two origins |
| Install + roundtrip | Integration | `cc-core/tests/` | Install a skill, read it back, assert equality |
| CLI output format | E2E | `cc-cli/tests/` | Run `cc list skills --format json`, parse stdout |
| Error messages | E2E | `cc-cli/tests/` | Run `cc add skill nonexistent`, assert stderr |
| Validate | E2E | `cc-cli/tests/` | Run `cc validate` on fixture project, assert exit code |

### Key test dependencies

```toml
[dev-dependencies]
tempfile = "3"          # Temporary directories for integration tests
assert_cmd = "2"        # CLI end-to-end testing
predicates = "3"        # Assertions for assert_cmd output
assert_fs = "1"         # Filesystem fixtures
```

---

## 7. Deployment

### Build and distribution

| Aspect | Approach |
|--------|----------|
| **Build** | `cargo build --release` → single static binary (~5-10MB) |
| **Targets** | `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `aarch64-unknown-linux-gnu` |
| **Distribution** | GitHub Releases with cross-compiled binaries |
| **Install** | Download binary → add to `PATH`; future: `cargo install convenient-claude` |
| **Auto-update** | Future: check GitHub Releases for newer version on `cc doctor` |

### CI pipeline

```
on push/PR:
  ├── cargo fmt --check
  ├── cargo clippy --deny warnings
  ├── cargo test --all
  └── cargo build --release (matrix: linux, macos, windows)

on tag v*:
  └── build release binaries
      └── upload to GitHub Release
          └── generate changelog from conventional commits
```

### Distribution comparison

| Method | Pros | Cons |
|--------|------|------|
| GitHub Release binaries | Zero dependencies, instant download | Manual updates |
| `cargo install` | Familiar to Rust users | Requires Rust toolchain |
| Homebrew tap | Easy for macOS users | Maintenance overhead |
| **Recommendation** | **Start with GitHub Releases.** Add `cargo install` and Homebrew as adoption grows. | |

---

## 8. Use Cases

### UC-1: First-time project setup

```
$ cd my-new-project
$ cc init

  Initialized .claude/ configuration in my-new-project/

  Created:
    .claude/settings.json       (empty permissions)
    .claude/skills/             (0 skills)
    .claude/commands/           (0 commands)
    .claude/agents/             (0 agents)
    .claude/rules/              (0 rules)

  Next steps:
    cc list skills --from extern   # browse available skills
    cc add skill <name>            # install what you need
```

### UC-2: Discover and install from external library

```
$ cc list skills --from extern/claude-skills

  Skills from extern/claude-skills (66 available):

  Name               Domain           Description
  react-expert       frontend         React/Next.js specialist...
  python-pro         language         Python expert with...
  api-architect      api-architecture REST/GraphQL design...

$ cc add skill react-expert --from extern/claude-skills

  ✓ Installed skill "react-expert" → .claude/skills/react-expert.md
    Version: 1.0.0 | Domain: frontend | Author: Jeffallan

$ cc add skill python-pro --from extern/claude-skills

  ✓ Installed skill "python-pro" → .claude/skills/python-pro.md
```

### UC-3: Diagnose configuration issues

```
$ cc doctor

  Checking Claude Code configuration...

  ✓ .claude/ directory exists
  ✓ settings.json is valid JSON
  ⚠ settings.json: 'Bash(cargo build)' should be 'Bash(cargo build:*)'
    → Use --fix to auto-correct
  ✓ 2 skills installed, all valid
  ✓ 1 command installed, all valid
  ✓ 3 hooks registered, all valid
  ⚠ skill "react-expert" has missing field 'version' in metadata
  ✗ hook on "PreToolUse" references missing script "scripts/check.sh"

  Issues: 1 error, 2 warnings
  Run 'cc doctor --fix' to auto-fix warnings.
```

### UC-4: Session with specific resources

```
$ cc session start --mode interactive --skills commit,review

  Session started (interactive mode)

  Active skills:
    ● commit     (.claude/skills/commit.md)
    ● review     (.claude/skills/review.md)

  Active hooks:
    ● PreToolUse → lint-check.sh
    ● PostToolUse → format-check.sh

  Commands available:
    /commit, /review, /validate

  Session config written to .claude/session.json

  [Running interactive session — use 'cc session stop' to end]

$ cc session status

  Session: active
  Mode: interactive
  Skills: 2 active (commit, review)
  Hooks: 2 active
  Duration: 12m
  Tokens:  48,230 in / 12,100 out (60,330 total)
  Cost:    ~$0.18 (estimated)
```

### UC-5: Validate project resources

```
$ cc validate

  Validating .claude/ resources...

  Skills (3):
    ✓ commit        valid
    ✓ review        valid
    ✓ react-expert  valid

  Commands (2):
    ✓ deploy        valid
    ✗ test-all      missing required field 'description'

  Hooks (2):
    ✓ PreToolUse    valid
    ✓ PostToolUse   valid

  Rules (1):
    ✓ python.md     valid

  Settings:
    ⚠ duplicate permission entry: 'Bash(cargo build)'

  Result: 1 error, 1 warning
```

### UC-6: Sync external libraries

```
$ cc sync --dry-run

  Checking extern/ updates...

  extern/claude-skills:
    + 3 new skills:  rust-expert, kubernetes-pro, graphql-architect
    ~ 2 updated:     react-expert (1.0.0 → 1.1.0), python-pro (1.0.0 → 1.2.0)
    - 1 removed:     deprecated-old-skill

  extern/everything-claude-code:
    + 5 new commands: ...
    ~ 1 updated:     ...

  Run 'cc sync' to apply these changes.

$ cc sync

  Syncing extern/claude-skills... ✓
  Syncing extern/everything-claude-code... ✓

  8 new resources, 3 updated, 1 removed.
  Installed resources are NOT affected. Run 'cc list' to review.
```

### UC-7: Inspect a specific resource

```
$ cc show skill react-expert

  Skill: react-expert
  Origin: project (.claude/skills/react-expert.md)
  Source: installed from extern/claude-skills

  Metadata:
    Version:    1.0.0
    Author:     Jeffallan
    Domain:     frontend
    Role:       specialist
    Triggers:   react, nextjs, component, hook, jsx, tsx

  Description:
    React/Next.js specialist for component architecture,
    hooks patterns, and performance optimization.

  Related skills: typescript-expert, nextjs-pro

  Body preview (first 10 lines):
    ---
    # React Expert Skill
    ...
```

### UC-8: Manage configuration values

```
$ cc config show

  Effective settings (merged):

  Source: user (~/.claude/settings.json)
    permissions.allow:
      • Bash(cargo build)
      • Bash(cargo test)

  Source: project (.claude/settings.json)
    permissions.allow:
      • Bash(cargo build:*)    ← overrides user
      • Bash(cargo clippy)
    hooks.PreToolUse:
      • lint-check.sh

$ cc config set permissions.allow "Bash(cargo build:*)" --scope project

  ✓ Set permissions.allow = "Bash(cargo build:*)" in project settings
```

### UC-9: Check session stats (live)

```
$ cc stats session

  Session: a3f2c1d0  |  Mode: interactive  |  Duration: 34m

  Token Usage:
    Input:      142,830
    Output:      38,210
    Total:      181,040
    Est. cost:  $0.54

  Per-resource breakdown:
    Resource          Invocations   Tokens    % of Total
    commit           12            54,200    29.9%
    review           8             41,300    22.8%
    react-expert     6             38,900    21.5%
    (base prompt)    34            46,640    25.8%

  Tool invocations:
    Bash         89
    Read         134
    Write        23
    Grep         67
    Glob         31
```

### UC-10: View historical session stats

```
$ cc stats history --last 5

  Recent sessions (last 5):

  Date          Mode         Duration   Tokens     Cost
  2026-04-22    interactive  34m        181,040    $0.54
  2026-04-22    loop         12m        92,300     $0.28
  2026-04-21    conversation 5m         18,400     $0.06
  2026-04-21    interactive  58m        340,200    $1.02
  2026-04-20    loop         22m        145,600    $0.44

  Totals (5 sessions):
    Duration:  2h 11m
    Tokens:    777,540
    Cost:      $2.34
```

### UC-11: Per-resource usage analysis

```
$ cc stats resources

  Resource usage across all sessions (last 30 days):

  Skills:
    Name             Sessions   Tokens      Avg/session   Cost
    commit           14         620,400     44,314        $1.86
    review           11         490,100     44,555        $1.47
    react-expert     8          310,200     38,775        $0.93
    python-pro       5          180,500     36,100        $0.54

  Agents:
    Name             Sessions   Tokens      Avg/session   Cost
    docs-researcher  6          245,000     40,833        $0.74

  Commands:
    Name             Invocations   Tokens     Cost
    /deploy          22            198,000    $0.59
    /test-all        18            145,200    $0.44
    /validate        31            62,000     $0.19

  Most expensive resource: commit ($1.86)
  Most used resource: /validate (31 invocations)
```

---

## Appendix A: Crate Summary

| Crate | Role | Key Dependencies |
|-------|------|-----------------|
| `cc-cli` | CLI entrypoint, dispatch, output | `cc-core`, `clap`, `anyhow`, `colored`, `comfy-table`, `dialoguer` |
| `cc-core` | Business logic, resolution, sessions | `cc-schema`, `walkdir`, `thiserror`, `tracing` |
| `cc-schema` | Data structures, parsing, I/O | `serde`, `serde_json`, `serde_yaml`, `pulldown-cmark`, `thiserror` |

## Appendix B: File Format Reference

| Resource | Format | Location | Schema complexity |
|----------|--------|----------|------------------|
| Skill | Markdown + YAML frontmatter | `.claude/skills/*.md` | High (10+ metadata fields) |
| Command | Markdown + YAML frontmatter | `.claude/commands/*.md` | Medium (name, description, tools) |
| Agent | Markdown + YAML frontmatter | `.claude/agents/*.md` | Medium (name, description, model, tools) |
| Hook | JSON (in settings.json) | `.claude/settings.json` | Medium (event → matcher → commands) |
| Rule | Plain Markdown | `.claude/rules/*.md` | Low (name from filename, body from content) |
| Settings | JSON | `.claude/settings.json` | Medium (permissions + hooks) |
| Session config | JSON | `.claude/session.json` | Low (ephemeral, mode + active resources) |
| Session stats | JSON | `.claude/session-stats.json` | Medium (tokens, resource usage, tool counts) |
| Stats history | JSONL | `.claude/stats-history.jsonl` | One SessionStats per line, append-only |
| CLAUDE.md | Plain Markdown | Project root | None (opaque text) |

## Appendix C: Competitive Landscape

| Tool | All resource types | Install lifecycle | Native binary | Session mgmt | Validation |
|------|-------------------|-------------------|---------------|-------------|------------|
| **convenient-claude** | **All 5** | **Full** | **Rust** | **Yes** | **Yes** |
| claude-cmd | Commands only | Install/search | No (Node.js) | No | No |
| caude-skill-manager | Skills only | Install | No | No | No |
| ccexp | All (read-only) | None (browse only) | No | No | No |
| claude-code-tool-manager | MCP + skills + agents | Import | Rust (Tauri GUI) | No | No |
| claude-config-composer | Generated only | One-shot | No | No | No |
