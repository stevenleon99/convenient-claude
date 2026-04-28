# cc HUD — Status Line for Claude Code

A native Rust implementation of a real-time status line for Claude Code, compiled into `cc.exe`. It displays session information in Claude Code's status bar, similar to `claude-hud` but with a simpler setup.

## Quick Start

### 1. Install the cc binary

Build from source or use a pre-built release:

```bash
cargo build --release
# Binary at: target/release/cc.exe
```

### 2. Configure Claude Code

Run the setup command to get the `settings.json` entry:

```bash
cc session setup-hud
```

Output:
```json
{
  "statusLine": {
    "type": "command",
    "command": "\"C:\\path\\to\\cc.exe\" session hud"
  }
}
```

Add this to `~/.claude/settings.json` (or merge into existing settings).

### 3. Restart Claude Code

The status line will appear in Claude Code's bottom bar, updating every ~300ms.

## Commands

### `cc session hud`

Run the HUD status line. Reads JSON from stdin (piped by Claude Code) and prints ANSI-colored output to stdout.

```bash
cc session hud [--layout <compact|expanded>]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--layout` | `expanded` | Display layout mode |

**Layout modes:**
- `expanded` — Multi-line display with tools/agents/todos activity
- `compact` — Single-line summary

This command is meant to be invoked by Claude Code via the `statusLine` setting, not manually.

### `cc session setup-hud`

Print the `settings.json` configuration snippet. Run this once to get the exact JSON to paste into your settings.

```bash
cc session setup-hud
```

## Display Elements

The status line shows:

| Element | Description |
|---------|-------------|
| **Model badge** | `[Sonnet 4.6]` — current model name, with provider label if applicable |
| **Context bar** | `████░░░░░░ 45%` — visual usage bar (green/yellow/red thresholds) |
| **Project path** | Last N segments of working directory |
| **Git status** | `git:(main*)` — branch, dirty marker, ahead/behind counts |
| **Config counts** | `1 CLAUDE.md`, `2 rules`, `3 MCPs`, `4 hooks` |
| **Usage limits** | `Usage ██░░░░░░░ 25% (5h)` — rate limit consumption |
| **Cost** | `$1.23` — session cost estimate |
| **Duration** | `⏱ 1h 30m` — session elapsed time |
| **Token breakdown** | `(in: 90k, cache: 5k)` — shown at high context (≥85%) |

### Context Bar Colors

| Threshold | Color |
|-----------|-------|
| 0–59% | Green |
| 60–84% | Yellow |
| 85–100% | Red |

### Usage Warning

When rate limits reach 100%, a warning appears:

```
⚠ Limit reached
```

## Architecture

### Data Flow

```
Claude Code (every ~300ms)
    │
    │  JSON via stdin
    │  { model, context_window, cwd, rate_limits, cost, transcript_path }
    │
    ▼
cc session hud
    │
    │  1. Parse stdin JSON
    │  2. Parse transcript JSONL (tools, agents, todos)
    │  3. Count configs (CLAUDE.md, rules, MCPs, hooks)
    │  4. Get git status
    │  5. Render ANSI output
    │
    ▼
ANSI status lines (stdout)
    │
    ▼
Claude Code status bar
```

### Source Files

| Module | Purpose |
|--------|---------|
| `crates/cc-hud/src/types.rs` | Data structures for stdin/transcript/render context |
| `crates/cc-hud/src/stdin.rs` | Timeout-based stdin reader + extraction helpers |
| `crates/cc-hud/src/transcript.rs` | JSONL transcript parser |
| `crates/cc-hud/src/config_count.rs` | Count CLAUDE.md/rules/MCPs/hooks |
| `crates/cc-hud/src/git.rs` | Git status via CLI |
| `crates/cc-hud/src/render/` | ANSI rendering (colors, bars, lines) |

## Comparison vs claude-hud

| Aspect | cc HUD | claude-hud |
|--------|--------|------------|
| **Runtime** | Native Rust binary | Node.js / TypeScript |
| **Setup command** | `cc session setup-hud` | `/claude-hud:setup` |
| **settings.json command** | `"cc.exe" session hud` (20 chars) | 300+ char bash one-liner |
| **Cold start** | ~1ms | ~50ms (Node startup) |
| **Dependencies** | Single binary | Node + npm + plugin cache |
| **Platform** | Windows native | Bash required |

## Troubleshooting

### No output appears

- Verify `statusLine` is correctly set in `~/.claude/settings.json`
- Check the path to `cc.exe` is correct (Windows paths need double backslashes in JSON)
- Run `cc session hud` manually with sample JSON to test

### Git status not showing

- Ensure `git` is installed and available in PATH
- Git commands run in the `cwd` directory from stdin

### Colors not rendering

- Set `TERM` environment variable if needed
- Use `--no-color` flag to disable if terminal doesn't support ANSI

## Example Output

**Normal session:**
```
[Sonnet 4.6] ████░░░░░░ 45% │ convenient-claude git:(main*) │ 1 CLAUDE.md │ Usage ██░░░░░░░ 25% (5h) │ $1.23
```

**High context:**
```
[Opus 4.6] ████████░░ 88% │ my-project git:(main*) │ 2 CLAUDE.md │ ⚠ Limit reached │ $3.45 │ (in: 176k, cache: 8k)
```

**Compact layout:**
```
[Sonnet 4.6] ████░░░░░░ 45% │ my-project git:(main*) │ $1.23 │ ⏱ 1h 30m
```

## Customization

Future versions may support a config file at `~/.claude/cc-hud.json` for:

```json
{
  "layout": "expanded",
  "showTools": true,
  "showAgents": true,
  "showTodos": true,
  "showUsage": true,
  "showCost": true,
  "showDuration": true,
  "contextBarWidth": 10,
  "pathLevels": 1
}
```

Currently, the default configuration shows all elements.