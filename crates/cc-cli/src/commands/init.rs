use crate::output;
use anyhow::{bail, Result};
use std::io::Write;

pub fn run(project_dir: &std::path::Path, workspace_root: &std::path::Path) -> Result<()> {
    match cc_core::init_project(project_dir, workspace_root) {
        Ok(cc_core::InitResult::Created { items }) => {
            output::print_success(&format!(
                "Initialized .claude/ configuration in {}",
                project_dir.display()
            ));
            println!();
            println!("Created:");
            for item in &items {
                println!("  {item}");
            }
            print_next_steps();
        }
        Ok(cc_core::InitResult::AlreadyExists { existing }) => {
            handle_existing(project_dir, workspace_root, &existing)?;
        }
        Err(e) => bail!("Init failed: {e}"),
    }
    Ok(())
}

fn handle_existing(
    project_dir: &std::path::Path,
    workspace_root: &std::path::Path,
    existing: &cc_core::ExistingClaudeDir,
) -> Result<()> {
    output::print_info(&format!(
        ".claude/ directory already exists in {}",
        project_dir.display()
    ));
    println!();

    // Show what's there
    println!("Current contents:");
    println!(
        "  settings.json: {}",
        if existing.has_settings { "present" } else { "missing" }
    );
    println!("  skills:   {} file(s)", existing.skills_count);
    println!("  commands: {} file(s)", existing.commands_count);
    println!("  agents:   {} file(s)", existing.agents_count);
    println!("  rules:    {} file(s)", existing.rules_count);
    println!("  hooks:    {} hook(s)", existing.hooks_count);
    println!();

    // Present options
    println!("Choose an option:");
    println!("  [1] Re-initialize (restore missing dirs/files, keep existing resources)");
    println!("  [2] Force fresh init (delete .claude/ and recreate)");
    println!("  [3] Cancel");
    println!();

    loop {
        print!("Enter choice (1-3): ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        match choice {
            "1" => {
                match cc_core::reinit_project(project_dir, workspace_root) {
                    Ok(created) => {
                        output::print_success("Re-initialized .claude/ configuration");
                        if !created.is_empty() {
                            println!();
                            println!("Created:");
                            for item in &created {
                                println!("  {item}");
                            }
                        } else {
                            println!();
                            output::print_info("All files and directories already present — nothing to create");
                        }
                        print_next_steps();
                    }
                    Err(e) => bail!("Re-init failed: {e}"),
                }
                break;
            }
            "2" => {
                let total = existing.total_resources();
                if total > 0 {
                    println!();
                    output::print_error(&format!(
                        "Warning: This will delete {total} existing resource(s)!",
                    ));
                    print!("Type 'yes' to confirm: ");
                    std::io::stdout().flush()?;
                    let mut confirm = String::new();
                    std::io::stdin().read_line(&mut confirm)?;
                    if confirm.trim() != "yes" {
                        println!("Cancelled.");
                        break;
                    }
                }

                match cc_core::force_init_project(project_dir, workspace_root) {
                    Ok(created) => {
                        output::print_success("Force-initialized fresh .claude/ configuration");
                        println!();
                        println!("Created:");
                        for item in &created {
                            println!("  {item}");
                        }
                        print_next_steps();
                    }
                    Err(e) => bail!("Force init failed: {e}"),
                }
                break;
            }
            "3" | "" => {
                println!("Cancelled.");
                break;
            }
            _ => {
                output::print_error("Invalid choice. Enter 1, 2, or 3.");
                continue;
            }
        }
    }

    Ok(())
}

fn print_next_steps() {
    println!();
    output::print_info("Project registered in cc-workspace.toml");
    println!();
    println!("Next steps:");
    println!("  cc tui                      # interactive dashboard to browse & install");
    println!("  cc list skills              # list available skills");
    println!("  cc add skill <name>         # install a specific skill");
}
