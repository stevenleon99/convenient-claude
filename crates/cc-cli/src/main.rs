mod args;
mod commands;
mod output;
mod tui;

use anyhow::Result;
use clap::Parser;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let cli = args::Cli::parse();

    // Suppress colors if requested
    if cli.no_color {
        owo_colors::set_override(false);
    }

    // The app dir is where resource/cc-workspace.toml lives — found via CC_HOME, binary location, or CWD.
    let app_dir = resolve_app_dir()?;

    // The project dir comes from --project-dir, .claude/ detection, or cwd.
    let project_dir = resolve_project_dir(cli.project_dir.as_deref(), &app_dir)?;

    match cli.command {
        args::Commands::Init => {
            // init always targets the current working directory, regardless of what's in toml
            let cwd = std::env::current_dir()?;
            commands::init::run(&cwd, &app_dir)?;
        }

        args::Commands::List { resource } => {
            commands::list::run(&resource, &project_dir, &app_dir)?
        }

        args::Commands::Add { resource } => {
            commands::add::run(&resource, &project_dir, &app_dir)?
        }

        args::Commands::Remove { resource } => {
            commands::remove::run(&resource, &project_dir)?
        }

        args::Commands::Show {
            resource_type,
            name,
        } => commands::show::run(&resource_type, &name, &project_dir)?,

        args::Commands::Validate { fix } => {
            commands::validate::run(&project_dir, fix, &app_dir)?
        }

        args::Commands::Session { action } => commands::session::run(&action, &project_dir, &app_dir)?,

        args::Commands::Config { action } => {
            commands::stats_cmd::run_config(&action, &project_dir)?
        }

        args::Commands::Stats { action } => commands::stats_cmd::run_stats(&action, &project_dir)?,

        args::Commands::Doctor => commands::doctor::run(&project_dir, &app_dir)?,

        args::Commands::Tui => commands::tui::run(&project_dir, &app_dir)?,
    }

    Ok(())
}

/// Find the cc application directory:
/// 1. CC_HOME env var if set
/// 2. Relative to the running binary (parent dirs containing resource/cc-workspace.toml)
/// 3. Walk up from CWD as fallback (current behavior)
fn resolve_app_dir() -> Result<PathBuf> {
    // 1. CC_HOME override
    if let Ok(cc_home) = std::env::var("CC_HOME") {
        let path = PathBuf::from(&cc_home);
        if path.join("resource").join("cc-workspace.toml").exists() {
            return Ok(path);
        }
        anyhow::bail!("CC_HOME set to '{}' but no cc-workspace.toml found there", cc_home);
    }

    // 2. Search relative to the binary
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let mut dir = exe_dir.to_path_buf();
            loop {
                if dir.join("resource").join("cc-workspace.toml").exists() {
                    return Ok(dir);
                }
                if !dir.pop() { break; }
            }
        }
    }

    // 3. Fallback: walk up from CWD (preserves current behavior)
    let cwd = std::env::current_dir()?;
    let mut dir = cwd.clone();
    loop {
        if dir.join("resource").join("cc-workspace.toml").exists() {
            return Ok(dir);
        }
        if !dir.pop() {
            return Ok(cwd);
        }
    }
}

/// Resolve the project directory:
/// 1. If --project-dir is given, use that.
/// 2. Try to find project directory by searching upward for .claude/
/// 3. Fall back to cwd.
fn resolve_project_dir(override_dir: Option<&str>, _app_dir: &Path) -> Result<PathBuf> {
    if let Some(dir) = override_dir {
        let path = PathBuf::from(dir);
        if path.is_dir() {
            return Ok(path);
        }
        anyhow::bail!("Specified project directory does not exist: {dir}");
    }

    // Try to find project directory by searching upward for .claude/
    let cwd = std::env::current_dir()?;
    if let Some(project_dir) = cc_core::paths::find_project_dir(&cwd) {
        return Ok(project_dir);
    }

    // Default to cwd
    Ok(cwd)
}

/// Strip Windows verbatim path prefix `\\?\` if present.
#[allow(dead_code)]
fn strip_verbatim_prefix(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    } else {
        path.to_path_buf()
    }
}
