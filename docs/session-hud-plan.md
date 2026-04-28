# cc session hud — Implementation Plan

## Overview

Overhaul `cc session` to produce a **real-time status line** for Claude Code's status bar, replicating what `claude-hud` does — but as a native Rust binary compiled into `cc.exe`. After the code changes, running `cc session setup-hud` will print the one-line command for `~/.claude/settings.json`.

## Feasibility Assessment

**Verdict: Fully feasible.** Here's why:

| Aspect | Assessment |
|--------|-----------|
| **Data source** | Claude Code pipes JSON to stdin of the `statusLine` command. The schema is well-defined (`StdinData` in claude-hud). Rust can read and parse this trivially via `serde_json`. |
| **Transcript parsing** | claude-hud reads a JSONL transcript file to extract tools, agents, todos, token counts. Rust has excellent JSONL/line-by-line parsing. A disk-based cache avoids re-parsing. |
| **ANSI rendering** | The output is colored ANSI text. Rust crates like `console`, `owo-colors` (already in-tree), or raw escape sequences handle this. |
| **Performance** | Rust will be faster than the current TypeScript/Node.js claude-hud. Cold start is near-instant. No runtime to locate. |
| **Cross-platform** | Already targeting Windows. Terminal width detection needs platform-aware code (Windows vs Unix). |
| **Single binary** | `cc.exe` is already compiled. The hud subcommand adds ~30KB. No external deps needed at runtime. |
| **settings.json integration** | The one-liner is simpler for Rust: `cc.exe session hud` instead of the complex bash/ts-node dance. |

### Key Advantage Over claude-hud

The current claude-hud `settings.json` command is a 300+ character bash one-liner that:
- Detects terminal width via `stty`
- Finds the plugin cache directory
- Sorts version directories
- Invokes a Node.js runtime on TypeScript source

Replacing it with `cc.exe session hud` is a **single 20-character command** — no bash, no Node, no plugin discovery.

## Architecture

### Data Flow

```
Claude Code                     cc.exe session hud
    │                                  │
    │  JSON via stdin (every ~300ms)   │
    ├─────────────────────────────────►│
    │  {                               │  1. Parse stdin JSON
    │    model: {...},                  │  2. Read transcript JSONL
    │    context_window: {...},         │  3. Count configs (CLAUDE.md, etc.)
    │    transcript_path: "...",        │  4. Get git status
    │    rate_limits: {...},            │  5. Render ANSI lines
    │    cost: {...},                   │  6. Print to stdout
    │    cwd: "..."                     │
    │  }                               │
    │                                  │
    │  ANSI status lines (stdout)      │
    │◄─────────────────────────────────┤
```

### New Crate: `cc-hud`

Add `crates/cc-hud/` as a new workspace member. This isolates the HUD logic from the CLI and core crates, keeping concerns separated.

```
crates/cc-hud/
├── Cargo.toml
└── src/
    ├── lib.rs           — Public API: render_status_line()
    ├── stdin.rs         — Read & parse JSON from stdin (StdinData struct)
    ├── types.rs         — StdinData, TranscriptData, ToolEntry, etc.
    ├── transcript.rs    — Parse JSONL transcript file + caching
    ├── config_count.rs  — Count CLAUDE.md, rules, MCPs, hooks
    ├── git.rs           — Get git branch/status
    ├── render/
    │   ├── mod.rs       — Top-level render dispatcher
    │   ├── colors.rs    — ANSI color helpers
    │   ├── session.rs   — Session line (model, context bar, usage)
    │   ├── tools.rs     — Tool activity line
    │   ├── agents.rs    — Agent tracking line
    │   ├── todos.rs     — Todo progress line
    │   ├── terminal.rs  — Terminal width, bar drawing, wrapping
    │   └── lines/       — Individual line renderers
    │       ├── identity.rs   — Context bar
    │       ├── project.rs    — Project path
    │       ├── usage.rs      — Rate limits
    │       ├── environment.rs — Config counts
    │       └── cost.rs       — Cost estimate
    └── config.rs        — HUD display config (what to show/hide)
```

### Modified Files

| File | Change |
|------|--------|
| `Cargo.toml` (workspace) | Add `cc-hud` to workspace members |
| `crates/cc-cli/Cargo.toml` | Add `cc-hud` dependency |
| `crates/cc-cli/src/args.rs` | Add `Hud` variant to `SessionAction` |
| `crates/cc-cli/src/commands/session.rs` | Add `hud()` dispatch + `setup_hud()` command |

## Implementation Steps

### Phase 1: Foundation (types + stdin)

**Step 1.1 — Create `cc-hud` crate skeleton**

- `crates/cc-hud/Cargo.toml` with deps: `serde`, `serde_json`, `chrono`, `uuid`, `console` (or use in-tree `owo-colors`), `git2` (or shell out to `git`)
- `crates/cc-hud/src/lib.rs` — re-export modules
- `crates/cc-hud/src/types.rs` — Port the TypeScript types from `claude-hud/src/types.ts`:

```rust
/// Mirrors claude-hud StdinData — the JSON Claude Code pipes via stdin.
#[derive(Deserialize, Default)]
pub struct StdinData {
    pub transcript_path: Option<String>,
    pub cwd: Option<String>,
    pub model: Option<ModelInfo>,
    pub context_window: Option<ContextWindow>,
    pub cost: Option<CostInfo>,
    pub rate_limits: Option<RateLimits>,
    pub effort: Option<EffortInfo>,
}

#[derive(Deserialize)]
pub struct ModelInfo {
    pub id: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Deserialize)]
pub struct ContextWindow {
    pub context_window_size: Option<u64>,
    pub total_input_tokens: Option<u64>,
    pub current_usage: Option<TokenUsage>,
    pub used_percentage: Option<f64>,
}

#[derive(Deserialize)]
pub struct TokenUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cache_creation_input_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
}

#[derive(Deserialize)]
pub struct CostInfo {
    pub total_cost_usd: Option<f64>,
    pub total_duration_ms: Option<u64>,
}

#[derive(Deserialize)]
pub struct RateLimits {
    pub five_hour: Option<RateWindow>,
    pub seven_day: Option<RateWindow>,
}

#[derive(Deserialize)]
pub struct RateWindow {
    pub used_percentage: Option<f64>,
    pub resets_at: Option<f64>,  // Unix timestamp
}

// Transcript types
pub struct TranscriptData {
    pub tools: Vec<ToolEntry>,
    pub agents: Vec<AgentEntry>,
    pub todos: Vec<TodoItem>,
    pub session_start: Option<chrono::DateTime<chrono::Utc>>,
    pub session_name: Option<String>,
    pub session_tokens: Option<SessionTokenUsage>,
    pub last_compact_boundary_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_compact_post_tokens: Option<u64>,
}
```

**Step 1.2 — stdin reader (`crates/cc-hud/src/stdin.rs`)**

Port the timeout-based stdin reader from `claude-hud/src/stdin.ts`:

```rust
pub fn read_stdin(timeout: Duration) -> Result<Option<StdinData>> {
    // Read all of stdin with a timeout (first byte: 250ms, idle: 30ms)
    // Parse as JSON into StdinData
    // If stdin is a TTY, return None
}
```

### Phase 2: Transcript Parser

**Step 2.1 — `crates/cc-hud/src/transcript.rs`**

Port `claude-hud/src/transcript.ts`:

- Read the JSONL file line by line
- Extract `tool_use` / `tool_result` pairs for tool tracking
- Extract `Task`/`Agent` blocks for agent tracking
- Extract `TodoWrite` / `TaskCreate` / `TaskUpdate` for todos
- Accumulate token usage from `assistant` messages
- Track `compact_boundary` system entries
- Implement file-based caching (hash the transcript path, store mtime+size+parsed data)

### Phase 3: Config & Git

**Step 3.1 — Config counting (`crates/cc-hud/src/config_count.rs`)**

Port `claude-hud/src/config-reader.ts`:

- Count CLAUDE.md files (project + user)
- Count rules in `.claude/settings.json` and project settings
- Count MCPs in settings
- Count hooks in settings

**Step 3.2 — Git status (`crates/cc-hud/src/git.rs`)**

Two options (recommend option A):
- **Option A**: Shell out to `git` (fast, simple, matches claude-hud approach)
- **Option B**: Use `git2` crate (no external dep, but heavier compile)

```rust
pub struct GitStatus {
    pub branch: String,
    pub is_dirty: bool,
    pub ahead: u32,
    pub behind: u32,
}

pub fn get_git_status(cwd: &Path) -> Option<GitStatus> {
    // git rev-parse --abbrev-ref HEAD
    // git status --porcelain
    // git rev-list --left-right --count @{upstream}...HEAD
}
```

### Phase 4: Renderer

**Step 4.1 — Color & terminal utilities (`crates/cc-hud/src/render/colors.rs`, `terminal.rs`)**

- ANSI escape code helpers (re-use `owo-colors` from the workspace)
- Terminal width detection (Windows: `GetConsoleScreenBufferInfo`, Unix: `ioctl TIOCGWINSZ` or `COLUMNS` env)
- Visual length calculation (handle wide chars, strip ANSI for measurement)
- Progress bar rendering

**Step 4.2 — Session line (`crates/cc-hud/src/render/session.rs`)**

Port `claude-hud/src/render/session-line.ts`. This is the main line showing:

```
[Opus 4.6] █████░░░░░ 45% | my-project git:(main*) | 2 CLAUDE.md | Usage ██░░░░░░░ 25% | ⏱ 1h 30m
```

Key rendering logic:
- Model name display + provider label
- Context bar with color thresholds (green/yellow/red)
- Project path (last N segments)
- Git status (branch, dirty, ahead/behind)
- Config counts
- Usage rate limits with bars
- Session duration
- Cost estimate

**Step 4.3 — Activity lines (tools, agents, todos)**

Port the three activity renderers:
- `tools.rs` — Show running/completed tools with file targets
- `agents.rs` — Show subagent status with model and description
- `todos.rs` — Show task completion progress

**Step 4.4 — Layout engine (`crates/cc-hud/src/render/mod.rs`)**

Port `claude-hud/src/render/index.ts`:
- Compact mode: single session line
- Expanded mode: multiple lines with configurable element order
- Line wrapping to terminal width
- Merge groups (combine elements on one line if they fit)

### Phase 5: CLI Integration

**Step 5.1 — Add `Hud` and `SetupHud` actions to args.rs**

```rust
#[derive(Subcommand)]
pub enum SessionAction {
    // ... existing: Start, Stop, Status, Stats
    /// Run the HUD status line (reads from stdin, prints ANSI to stdout).
    Hud {
        /// Layout: compact or expanded.
        #[arg(long, default_value = "expanded")]
        layout: String,
    },
    /// Print the settings.json statusLine command.
    SetupHud,
}
```

**Step 5.2 — Implement `session hud` command**

```rust
fn run_hud(layout: &str) -> Result<()> {
    let stdin_data = cc_hud::stdin::read_stdin(Duration::from_millis(250))?;
    let Some(data) = stdin_data else {
        return Ok(()); // No stdin, nothing to display
    };

    let transcript = cc_hud::transcript::parse(&data.transcript_path)?;
    let (claude_md, rules, mcps, hooks) = cc_hud::config_count::count(&data.cwd)?;
    let git = cc_hud::git::get_git_status(data.cwd.as_deref())?;

    let ctx = cc_hud::RenderContext {
        stdin: data,
        transcript,
        claude_md_count: claude_md,
        rules_count: rules,
        mcp_count: mcps,
        hooks_count: hooks,
        git_status: git,
        config: cc_hud::Config::default(),
    };

    let lines = cc_hud::render::render(&ctx);
    for line in lines {
        println!("{line}");
    }
    Ok(())
}
```

**Step 5.3 — Implement `session setup-hud` command**

This is the **settings.json one-liner generator**:

```rust
fn setup_hud() -> Result<()> {
    let exe_path = std::env::current_exe()?;
    let exe_str = exe_path.to_string_lossy();

    // On Windows, use forward slashes in the JSON
    let exe_escaped = exe_str.replace('\\', "\\\\");

    let command = format!("\"{exe_escaped}\" session hud");

    let settings_entry = serde_json::json!({
        "statusLine": {
            "type": "command",
            "command": command
        }
    });

    println!("Add this to ~/.claude/settings.json:");
    println!();
    println!("{}", serde_json::to_string_pretty(&settings_entry)?);
    Ok(())
}
```

Output:
```json
{
  "statusLine": {
    "type": "command",
    "command": "\"C:\\Users\\Steve\\.cargo\\bin\\cc.exe\" session hud"
  }
}
```

Compare to the claude-hud one-liner:
```bash
bash -c 'cols=$(stty size </dev/tty 2>/dev/null | awk "{print \\$2}"); export COLUMNS=$(( ${cols:-120} > 4 ? ${cols:-120} - 4 : 1 )); plugin_dir=$(ls -d "${CLAUDE_CONFIG_DIR:-$HOME/.claude}"/plugins/cache/*/claude-hud/*/ 2>/dev/null | awk -F/ "{ print $(NF-1) \"\\t\" $(0) }" | grep -E "^[0-9]+\\.[0-9]+\\.[0-9]+\\t" | sort -t. -k1,1n -k2,2n -k3,3n -k4,4n | tail -1 | cut -f2-); exec "{RUNTIME_PATH}" "${plugin_dir}src/index.ts"'
```

The Rust version is orders of magnitude simpler.

### Phase 6: Polish

**Step 6.1 — HUD config file support**

Support `~/.claude/cc-hud.json` or similar for display preferences:
```json
{
  "layout": "expanded",
  "showTools": true,
  "showAgents": true,
  "showTodos": true,
  "showUsage": true,
  "showCost": true,
  "contextBarWidth": 10,
  "pathLevels": 1
}
```

**Step 6.2 — Performance optimization**

- Cache transcript parsing results to disk (hash + mtime check)
- Minimize allocations in the render loop
- The entire stdin-read → parse → render → print pipeline should complete in <5ms

**Step 6.3 — Tests**

- Unit tests for stdin parsing (various StdinData shapes)
- Unit tests for transcript parser
- Unit tests for render output (snapshot testing)
- Integration test: pipe JSON to `cc session hud`, capture ANSI output

## Dependency Plan

### New dependencies for `cc-hud`

| Crate | Purpose | Notes |
|-------|---------|-------|
| `serde` / `serde_json` | JSON parsing | Already in workspace |
| `chrono` | Timestamp handling | Already in workspace |
| `owo-colors` | ANSI colors | Already in workspace |
| `dirs` | Find home/config dirs | Lightweight |

No heavy new dependencies needed. `git2` is optional (can shell out to `git` instead).

## Estimated Scope

| Component | Lines of Code (Rust) | Complexity |
|-----------|---------------------|------------|
| types.rs | ~150 | Low (1:1 port from TS) |
| stdin.rs | ~100 | Medium (timeout-based read) |
| transcript.rs | ~400 | High (JSONL parsing + caching) |
| config_count.rs | ~100 | Low (file counting) |
| git.rs | ~60 | Low (shell out to git) |
| render/ | ~600 | High (port rendering logic) |
| CLI integration | ~80 | Low |
| **Total** | **~1,500** | |

## Risk & Mitigation

| Risk | Mitigation |
|------|-----------|
| Windows terminal width detection differs from Unix | Use `crossterm` or Windows API; fallback to `COLUMNS` env var |
| Stdin timeout behavior differs on Windows | Test thoroughly; use `std::io::Read` with non-blocking or thread-based timeout |
| Claude Code changes the stdin JSON schema | Keep fields optional; degrade gracefully on missing data |
| Transcript format changes | Cache version field; re-parse on version mismatch |
| Performance regression vs TypeScript | Rust will be faster; profile if needed |

## Execution Order

1. **Phase 1** (types + stdin) — Foundation, everything depends on this
2. **Phase 3.2** (git) — Quick win, no dependencies
3. **Phase 3.1** (config count) — Quick win
4. **Phase 2** (transcript) — Core data extraction
5. **Phase 4** (renderer) — Visual output
6. **Phase 5** (CLI integration) — Wire everything together
7. **Phase 6** (polish) — Config, tests, optimization

Phases 3.1, 3.2, and parts of Phase 2 can be done in parallel after Phase 1.
