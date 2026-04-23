use crate::args::{ConfigAction, StatsAction};
use crate::output;
use anyhow::{bail, Result};
use cc_schema::Origin;
use owo_colors::OwoColorize;
use std::path::Path;

pub fn run_config(action: &ConfigAction, project_dir: &Path) -> Result<()> {
    match action {
        ConfigAction::Show => config_show(project_dir),
        ConfigAction::Get { key } => config_get(key, project_dir),
        ConfigAction::Set { key, value, scope } => config_set(key, value, scope, project_dir),
        ConfigAction::Diff => config_diff(project_dir),
    }
}

pub fn run_stats(action: &StatsAction, project_dir: &Path) -> Result<()> {
    match action {
        StatsAction::Session => stats_session(project_dir),
        StatsAction::History { last } => stats_history(project_dir, *last),
        StatsAction::Resources => stats_resources(project_dir),
    }
}

fn config_show(project_dir: &Path) -> Result<()> {
    let user_settings = cc_core::config::load_user_settings()?;
    let project_settings = cc_core::config::load_project_settings(project_dir)?;
    let merged = cc_core::config::merge_settings(&user_settings, &project_settings);

    println!("{}", "Effective settings (merged):".bold());
    println!();
    println!("{}", serde_json::to_string_pretty(&merged)?);
    Ok(())
}

fn config_get(key: &str, project_dir: &Path) -> Result<()> {
    let settings = cc_core::config::load_project_settings(project_dir)?;

    match key {
        "permissions.allow" => {
            for entry in &settings.permissions.allow {
                println!("  {entry}");
            }
        }
        "permissions.deny" => {
            for entry in &settings.permissions.deny {
                println!("  {entry}");
            }
        }
        _ => bail!("Unknown config key: '{key}'"),
    }
    Ok(())
}

fn config_set(key: &str, value: &str, scope: &str, project_dir: &Path) -> Result<()> {
    let origin = match scope {
        "user" => Origin::User,
        _ => Origin::Project,
    };

    cc_core::config::set_config_value(project_dir, key, value, &origin)?;
    output::print_success(&format!("Set {key} = \"{value}\" in {scope} settings"));
    Ok(())
}

fn config_diff(project_dir: &Path) -> Result<()> {
    let user = cc_core::config::load_user_settings()?;
    let project = cc_core::config::load_project_settings(project_dir)?;

    println!("{}", "User settings (~/.claude/settings.json):".bold());
    println!("{}", serde_json::to_string_pretty(&user)?);
    println!();
    println!("{}", "Project settings (.claude/settings.json):".bold());
    println!("{}", serde_json::to_string_pretty(&project)?);
    Ok(())
}

fn stats_session(project_dir: &Path) -> Result<()> {
    let stats = cc_core::session::load_session_stats(project_dir)
        .ok_or_else(|| anyhow::anyhow!("No active session. Start one with 'cc session start'."))?;

    output::print_session_stats(&stats);
    Ok(())
}

fn stats_history(project_dir: &Path, last: usize) -> Result<()> {
    let history = cc_core::stats::load_stats_history(project_dir);

    if history.is_empty() {
        println!("No session history found.");
        return Ok(());
    }

    let sessions: Vec<_> = history.into_iter().rev().take(last).collect();
    println!("{}", format!("Recent sessions (last {last}):").bold());
    println!();

    for session in &sessions {
        println!(
            "  {} | {:?} | tokens: {}",
            session.started_at.format("%Y-%m-%d %H:%M"),
            session.mode,
            session.token_usage.total_tokens,
        );
    }

    // Totals
    let total_tokens: u64 = sessions.iter().map(|s| s.token_usage.total_tokens).sum();
    println!();
    println!("Totals ({} sessions):", sessions.len());
    println!("  Tokens: {total_tokens}");

    Ok(())
}

fn stats_resources(project_dir: &Path) -> Result<()> {
    let usage = cc_core::stats::aggregate_resource_usage(project_dir, None);

    if usage.is_empty() {
        println!("No resource usage data found.");
        return Ok(());
    }

    println!("{}", "Resource usage (all sessions):".bold());
    println!();

    for ((rt, name), summary) in &usage {
        println!(
            "  {}/{}: {} sessions, {} tokens, {} invocations",
            rt, name, summary.total_sessions, summary.total_tokens, summary.total_invocations
        );
    }

    Ok(())
}
