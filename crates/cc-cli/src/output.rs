use cc_core::ResourceEntry;
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, Color as TableColor,
    ContentArrangement, Table,
};
use owo_colors::OwoColorize;
use std::io::Write;

/// Output format selection.
#[derive(Clone, Copy)]
pub enum Format {
    Table,
    Json,
    Plain,
}

/// Print a list of resource entries in the chosen format.
pub fn print_resource_list(entries: &[ResourceEntry], format: Format) {
    match format {
        Format::Table => print_resource_table(entries),
        Format::Json => print_resource_json(entries),
        Format::Plain => print_resource_plain(entries),
    }
}

fn print_resource_table(entries: &[ResourceEntry]) {
    if entries.is_empty() {
        println!("No resources found.");
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec!["Name", "Type", "Origin", "Active", "Description"]);

    for entry in entries {
        let active_str = if entry.active { "●" } else { "○" };
        let active_cell = if entry.active {
            Cell::new(active_str).fg(TableColor::Green)
        } else {
            Cell::new(active_str).fg(TableColor::DarkGrey)
        };

        table.add_row(vec![
            Cell::new(&entry.name),
            Cell::new(entry.resource_type.to_string()),
            Cell::new(entry.origin.to_string()),
            active_cell,
            Cell::new(entry.description.as_deref().unwrap_or("-")),
        ]);
    }

    println!("{table}");
}

fn print_resource_json(entries: &[ResourceEntry]) {
    let json_entries: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| {
            serde_json::json!({
                "name": e.name,
                "type": e.resource_type.to_string(),
                "origin": e.origin.to_string(),
                "active": e.active,
                "description": e.description,
            })
        })
        .collect();

    if let Ok(json) = serde_json::to_string_pretty(&json_entries) {
        println!("{json}");
    }
}

fn print_resource_plain(entries: &[ResourceEntry]) {
    for entry in entries {
        let active_marker = if entry.active { "*" } else { " " };
        println!(
            "{}{active_marker} {} ({}/{})",
            if entry.active { "" } else { "  " },
            entry.name,
            entry.resource_type,
            entry.origin,
        );
    }
}

/// Print validation findings.
pub fn print_findings(findings: &[cc_core::ValidationFinding]) {
    if findings.is_empty() {
        println!("{}", "All resources valid.".green());
        return;
    }

    for finding in findings {
        let level_str = match finding.level {
            cc_core::ValidationLevel::Error => "ERROR".red().to_string(),
            cc_core::ValidationLevel::Warning => "WARN".yellow().to_string(),
        };

        let location = finding
            .resource_name
            .as_deref()
            .or_else(|| finding.path.as_ref().and_then(|p| p.to_str()))
            .unwrap_or("unknown");

        println!("[{level_str}] {location}: {}", finding.message);
    }
}

/// Print session context info.
pub fn print_session_status(ctx: &cc_core::SessionContext) {
    println!("{} {}", "Session".bold(), ctx.session_id.dimmed());
    println!(
        "  Mode: {}",
        match ctx.mode {
            cc_schema::SessionMode::Conversation => "conversation",
            cc_schema::SessionMode::Loop => "loop",
            cc_schema::SessionMode::Interactive => "interactive",
        }
    );
    println!(
        "  Started: {}",
        ctx.started_at.format("%Y-%m-%d %H:%M:%S UTC")
    );

    if !ctx.active_skills.is_empty() {
        println!(
            "  Skills: {}",
            ctx.active_skills
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if !ctx.active_agents.is_empty() {
        println!(
            "  Agents: {}",
            ctx.active_agents
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if !ctx.active_commands.is_empty() {
        println!(
            "  Commands: {}",
            ctx.active_commands
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
}

/// Print session stats.
pub fn print_session_stats(stats: &cc_schema::SessionStats) {
    println!("{} {}", "Session Stats".bold(), stats.session_id.dimmed());
    println!(
        "  Tokens: {} in / {} out / {} total",
        stats.token_usage.input_tokens,
        stats.token_usage.output_tokens,
        stats.token_usage.total_tokens,
    );

    if let Some(cost) = stats.token_usage.estimated_cost {
        println!("  Estimated cost: ${cost:.4}");
    }

    if !stats.tool_invocations.is_empty() {
        println!("  Tool invocations:");
        let mut tools: Vec<_> = stats.tool_invocations.iter().collect();
        tools.sort_by(|a, b| b.1.cmp(a.1));
        for (tool, count) in tools {
            println!("    {tool}: {count}");
        }
    }

    if !stats.resource_usage.is_empty() {
        println!("  Resource usage:");
        for usage in &stats.resource_usage {
            println!(
                "    {}/{}: {} invocations, {} tokens",
                usage.resource_type, usage.name, usage.times_invoked, usage.tokens_consumed
            );
        }
    }
}

/// Print a success message.
pub fn print_success(msg: &str) {
    println!("{} {}", "✓".green(), msg);
}

/// Print an error message.
pub fn print_error(msg: &str) {
    let _ = std::io::stderr().write_all(format!("{} {}\n", "✗".red(), msg).as_bytes());
}

/// Print an info message.
pub fn print_info(msg: &str) {
    println!("{} {}", "→".blue(), msg);
}
