use crate::output;
use anyhow::Result;
use owo_colors::OwoColorize;
use std::path::Path;

pub fn run(project_dir: &Path) -> Result<()> {
    println!("Checking Claude Code configuration...\n");

    let mut issues = 0u32;
    let mut warnings = 0u32;

    // Check .claude/ directory
    let claude_dir = project_dir.join(".claude");
    if claude_dir.is_dir() {
        output::print_success(".claude/ directory exists");
    } else {
        output::print_error(".claude/ directory not found");
        issues += 1;
        println!("  → Run 'cc init' to create it");
        return Ok(());
    }

    // Check settings.json
    let settings_path = claude_dir.join("settings.json");
    if settings_path.exists() {
        match cc_schema::Settings::from_json(&std::fs::read_to_string(&settings_path)?) {
            Ok(settings) => {
                output::print_success("settings.json is valid JSON");
                if settings.permissions.allow.is_empty() && settings.permissions.deny.is_empty() {
                    println!("  {} No permissions configured", "→".blue());
                }
            }
            Err(e) => {
                output::print_error(&format!("settings.json parse error: {e}"));
                issues += 1;
            }
        }
    } else {
        println!(
            "  {} settings.json not found (will use defaults)",
            "⚠".yellow()
        );
        warnings += 1;
    }

    // Check resource directories
    for dir_name in &["skills", "commands", "agents", "rules"] {
        let dir = claude_dir.join(dir_name);
        if dir.is_dir() {
            let count = std::fs::read_dir(&dir)?.count();
            output::print_success(&format!("{dir_name}/ directory exists ({count} files)"));
        }
    }

    // Run validation
    let findings = cc_core::validate_project(project_dir);
    for finding in &findings {
        match finding.level {
            cc_core::ValidationLevel::Error => {
                output::print_error(&finding.message);
                issues += 1;
            }
            cc_core::ValidationLevel::Warning => {
                println!("  {} {}", "⚠".yellow(), finding.message);
                warnings += 1;
            }
        }
    }

    println!();
    if issues == 0 && warnings == 0 {
        output::print_success("All checks passed.");
    } else {
        println!("Issues: {issues} error(s), {warnings} warning(s)");
        if warnings > 0 {
            println!("Run 'cc doctor --fix' to auto-fix warnings.");
        }
    }

    Ok(())
}
