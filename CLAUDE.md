# convenient-claude

One stop place to setup Claude for your project including skills, commands, hooks, agents, and workflows.

## Project Overview

- **Language**: Rust (edition 2021)
- **Build**: `cargo build`
- **Test**: `cargo test`
- **Lint**: `cargo clippy`
- **Format**: `cargo fmt`

## Architecture

- `crates/cc-cli/` — Rust command line interface application source code
- `crates/cc-core/` — Rust core functionality library source code and will also serve as a foundation for shared logic and utilities
- `extern/` — External dependencies/submodules (e.g., claude-skills)

## Conventions

- Follow standard Rust conventions (rustfmt, clippy)
- Keep commands and skills in `.claude/commands/` and `.claude/skills/`
