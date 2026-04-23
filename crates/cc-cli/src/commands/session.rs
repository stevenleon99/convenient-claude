use crate::args::SessionAction;
use crate::output;
use anyhow::{bail, Result};
use std::path::Path;

pub fn run(action: &SessionAction, project_dir: &Path) -> Result<()> {
    match action {
        SessionAction::Start {
            mode,
            skills,
            agents,
        } => start_session(mode, skills, agents, project_dir),
        SessionAction::Stop => stop_session(project_dir),
        SessionAction::Status => session_status(project_dir),
        SessionAction::Stats => session_stats(project_dir),
    }
}

fn parse_mode(mode: &str) -> Result<cc_schema::SessionMode> {
    match mode.to_lowercase().as_str() {
        "conversation" => Ok(cc_schema::SessionMode::Conversation),
        "loop" => Ok(cc_schema::SessionMode::Loop),
        "interactive" => Ok(cc_schema::SessionMode::Interactive),
        _ => bail!("Unknown session mode: '{mode}'. Expected: conversation, loop, interactive"),
    }
}

fn start_session(
    mode: &str,
    skills: &[String],
    agents: &[String],
    project_dir: &Path,
) -> Result<()> {
    // Check if session already active
    if cc_core::session::load_session(project_dir).is_some() {
        bail!("Session already active. Run 'cc session stop' first.");
    }

    let session_mode = parse_mode(mode)?;
    let mut ctx = cc_core::SessionContext::new(session_mode);

    ctx.activate_skills(skills);
    ctx.activate_agents(agents);

    // Discover commands from project
    let extern_libs = cc_core::sync::list_extern_libs(project_dir);
    let commands =
        cc_core::discover_resources(cc_schema::ResourceType::Command, project_dir, &extern_libs);
    let active_cmds: Vec<String> = commands.iter().map(|c| c.name.clone()).collect();
    ctx.activate_commands(&active_cmds);

    // Save session
    cc_core::session::save_session(project_dir, &ctx)?;

    // Initialize stats
    let stats = cc_schema::SessionStats::new(ctx.session_id.clone(), session_mode);
    cc_core::session::save_session_stats(project_dir, &stats)?;

    output::print_success(&format!(
        "Session started ({})",
        match session_mode {
            cc_schema::SessionMode::Conversation => "conversation",
            cc_schema::SessionMode::Loop => "loop",
            cc_schema::SessionMode::Interactive => "interactive",
        }
    ));

    if !skills.is_empty() {
        println!("  Active skills: {}", skills.join(", "));
    }
    if !agents.is_empty() {
        println!("  Active agents: {}", agents.join(", "));
    }

    println!("\nSession config written to .claude/session.json");
    Ok(())
}

fn stop_session(project_dir: &Path) -> Result<()> {
    let _ctx = cc_core::session::load_session(project_dir)
        .ok_or_else(|| anyhow::anyhow!("No active session."))?;

    // Finalize and archive stats
    if let Some(mut stats) = cc_core::session::load_session_stats(project_dir) {
        stats.stop();
        cc_core::session::save_session_stats(project_dir, &stats)?;
        cc_core::session::append_to_history(project_dir, &stats)?;

        output::print_success("Session stopped.");
        output::print_session_stats(&stats);
    }

    cc_core::session::clear_session(project_dir)?;
    Ok(())
}

fn session_status(project_dir: &Path) -> Result<()> {
    let ctx = cc_core::session::load_session(project_dir)
        .ok_or_else(|| anyhow::anyhow!("No active session."))?;

    output::print_session_status(&ctx);

    if let Some(stats) = cc_core::session::load_session_stats(project_dir) {
        println!();
        output::print_session_stats(&stats);
    }

    Ok(())
}

fn session_stats(project_dir: &Path) -> Result<()> {
    let stats = cc_core::session::load_session_stats(project_dir)
        .ok_or_else(|| anyhow::anyhow!("No active session."))?;

    output::print_session_stats(&stats);
    Ok(())
}
