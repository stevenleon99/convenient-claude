mod args;
mod commands;
mod output;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

fn main() -> Result<()> {
    let cli = args::Cli::parse();

    // Suppress colors if requested
    if cli.no_color {
        owo_colors::set_override(false);
    }

    let project_dir = resolve_project_dir(cli.project_dir.as_deref())?;

    match cli.command {
        args::Commands::Init => commands::init::run(&project_dir)?,

        args::Commands::List { resource } => commands::list::run(&resource, &project_dir)?,

        args::Commands::Add { resource } => commands::add::run(&resource, &project_dir)?,

        args::Commands::Remove { resource } => commands::remove::run(&resource, &project_dir)?,

        args::Commands::Show {
            resource_type,
            name,
        } => commands::show::run(&resource_type, &name, &project_dir)?,

        args::Commands::Validate { fix } => commands::validate::run(&project_dir, fix)?,

        args::Commands::Sync { dry_run } => commands::sync::run(&project_dir, dry_run)?,

        args::Commands::Session { action } => commands::session::run(&action, &project_dir)?,

        args::Commands::Config { action } => {
            commands::stats_cmd::run_config(&action, &project_dir)?
        }

        args::Commands::Stats { action } => commands::stats_cmd::run_stats(&action, &project_dir)?,

        args::Commands::Doctor => commands::doctor::run(&project_dir)?,

        args::Commands::Completions { shell } => {
            eprintln!("Shell completions for '{shell}' are not yet implemented.");
            eprintln!("Use 'cc --help' for available commands.");
        }
    }

    Ok(())
}

/// Resolve the project directory from the given override or by searching upward.
fn resolve_project_dir(override_dir: Option<&str>) -> Result<PathBuf> {
    if let Some(dir) = override_dir {
        let path = PathBuf::from(dir);
        if path.is_dir() {
            return Ok(path.canonicalize()?);
        }
        anyhow::bail!("Specified project directory does not exist: {dir}");
    }

    // Try to find project directory by searching upward
    let cwd = std::env::current_dir()?;
    if let Some(project_dir) = cc_core::paths::find_project_dir(&cwd) {
        return Ok(project_dir);
    }

    // Default to cwd if no .claude/ found (commands like `init` work without it)
    Ok(cwd)
}
