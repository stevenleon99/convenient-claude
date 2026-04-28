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
        SessionAction::Hud { layout } => run_hud(layout),
        SessionAction::SetupHud => setup_hud(),
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
    let extern_libs = cc_core::list_extern_libs(project_dir);
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

fn run_hud(layout: &str) -> Result<()> {
    // Read JSON from stdin (piped by Claude Code every ~300ms)
    let stdin_data = match cc_hud::stdin::read_stdin()? {
        Some(data) => data,
        None => return Ok(()), // No stdin (not invoked by Claude Code)
    };

    // Parse transcript
    let transcript_path = stdin_data.transcript_path.as_deref().unwrap_or("");
    let transcript = cc_hud::transcript::parse_transcript(transcript_path);

    // Count configs
    let (claude_md, rules, mcps, hooks) =
        cc_hud::config_count::count_configs(stdin_data.cwd.as_deref());

    // Git status
    let git_status = cc_hud::git::get_git_status(stdin_data.cwd.as_deref());

    // Usage data
    let usage_data = cc_hud::stdin::get_usage_from_stdin(&stdin_data)
        .unwrap_or_default();

    // Layout
    let hud_layout = match layout.to_lowercase().as_str() {
        "compact" => cc_hud::types::Layout::Compact,
        _ => cc_hud::types::Layout::Expanded,
    };

    let mut config = cc_hud::types::HudConfig::default();
    config.layout = hud_layout;

    let ctx = cc_hud::types::RenderContext {
        stdin: stdin_data,
        transcript,
        claude_md_count: claude_md,
        rules_count: rules,
        mcp_count: mcps,
        hooks_count: hooks,
        git_status,
        usage_data,
        config,
    };

    let lines = cc_hud::render::render(&ctx);
    for line in lines {
        println!("{}", line);
    }

    Ok(())
}

fn setup_hud() -> Result<()> {
    let exe_path = std::env::current_exe()?;
    let exe_str = exe_path.to_string_lossy();

    // Escape backslashes for JSON on Windows
    let exe_escaped = exe_str.replace('\\', "\\\\");

    let command = format!("\"{}\" session hud", exe_escaped);

    println!("Add this to ~/.claude/settings.json:\n");

    let json = serde_json::json!({
        "statusLine": {
            "type": "command",
            "command": command
        }
    });

    println!("{}", serde_json::to_string_pretty(&json)?);

    println!("\nOr add just the statusLine entry to your existing settings.json:");
    println!("  \"statusLine\": {{ \"type\": \"command\", \"command\": \"\\\"{}\\\" session hud\" }}", exe_escaped);

    Ok(())
}
