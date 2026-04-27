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

    // The workspace root is where resource/cc-workspace.toml lives — search upward from cwd.
    let workspace_root = resolve_workspace_root()?;

    // The project dir comes from cc-workspace.toml's [current project], or --project-dir, or cwd.
    let project_dir = resolve_project_dir(cli.project_dir.as_deref(), &workspace_root)?;

    match cli.command {
        args::Commands::Init => commands::init::run(&project_dir, &workspace_root)?,

        args::Commands::List { resource } => {
            commands::list::run(&resource, &project_dir, &workspace_root)?
        }

        args::Commands::Add { resource } => {
            commands::add::run(&resource, &project_dir, &workspace_root)?
        }

        args::Commands::Remove { resource } => {
            commands::remove::run(&resource, &project_dir)?
        }

        args::Commands::Show {
            resource_type,
            name,
        } => commands::show::run(&resource_type, &name, &project_dir)?,

        args::Commands::Validate { fix } => {
            commands::validate::run(&project_dir, fix, &workspace_root)?
        }

        args::Commands::Sync { dry_run } => commands::sync::run(&project_dir, dry_run)?,

        args::Commands::Session { action } => commands::session::run(&action, &project_dir)?,

        args::Commands::Config { action } => {
            commands::stats_cmd::run_config(&action, &project_dir)?
        }

        args::Commands::Stats { action } => commands::stats_cmd::run_stats(&action, &project_dir)?,

        args::Commands::Doctor => commands::doctor::run(&project_dir, &workspace_root)?,

        args::Commands::Completions { shell } => {
            eprintln!("Shell completions for '{shell}' are not yet implemented.");
            eprintln!("Use 'cc --help' for available commands.");
        }

        args::Commands::Tui => commands::tui::run(&project_dir, &workspace_root)?,
    }

    Ok(())
}

/// Find the workspace root by searching upward for `resource/cc-workspace.toml`.
fn resolve_workspace_root() -> Result<PathBuf> {
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
/// 2. Otherwise read [current project] from cc-workspace.toml.
/// 3. Fall back to cwd.
fn resolve_project_dir(override_dir: Option<&str>, workspace_root: &Path) -> Result<PathBuf> {
    if let Some(dir) = override_dir {
        let path = PathBuf::from(dir);
        if path.is_dir() {
            return Ok(path);
        }
        anyhow::bail!("Specified project directory does not exist: {dir}");
    }

    // Try reading from cc-workspace.toml
    if let Some(config) = cc_core::WorkspaceConfig::load(workspace_root) {
        if let Some(project_path) = config.current_project_path(workspace_root) {
            // Strip Windows \\?\ prefix if present
            let clean_path = strip_verbatim_prefix(&project_path);
            if clean_path.is_dir() {
                return Ok(clean_path);
            }
        }
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
fn strip_verbatim_prefix(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    } else {
        path.to_path_buf()
    }
}
