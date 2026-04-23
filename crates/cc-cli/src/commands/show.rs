use crate::args::parse_resource_type;
use anyhow::{bail, Result};
use owo_colors::OwoColorize;
use std::path::Path;

pub fn run(resource_type: &str, name: &str, project_dir: &Path) -> Result<()> {
    let rt = parse_resource_type(resource_type)
        .ok_or_else(|| anyhow::anyhow!("Unknown resource type: '{resource_type}'"))?;

    match rt {
        cc_schema::ResourceType::Skill => {
            let (skill, origin) = cc_core::skill::load_skill(name, project_dir)?;
            println!("{}: {}", "Skill".bold(), skill.name);
            println!("  {}: {} ({})", "Origin".bold(), origin, origin);
            if let Some(ref path) = skill.source_path {
                println!("  {}: {}", "Path".bold(), path.display());
            }
            println!("  {}: {}", "Description".bold(), skill.description);
            if let Some(ref v) = skill.metadata.version {
                println!("  {}: {v}", "Version".bold());
            }
            if let Some(ref a) = skill.metadata.author {
                println!("  {}: {a}", "Author".bold());
            }
            if let Some(ref d) = skill.metadata.domain {
                println!("  {}: {d}", "Domain".bold());
            }
            if !skill.body.is_empty() {
                println!();
                println!("{}", "Body preview:".bold());
                let preview: String = skill.body.lines().take(10).collect::<Vec<_>>().join("\n");
                println!("{preview}");
            }
        }
        cc_schema::ResourceType::Command => {
            let (cmd, origin) = cc_core::command::load_command(name, project_dir)?;
            println!("{}: {}", "Command".bold(), cmd.name);
            println!("  {}: {} ({})", "Origin".bold(), origin, origin);
            println!("  {}: {}", "Description".bold(), cmd.description);
            if !cmd.allowed_tools.is_empty() {
                println!(
                    "  {}: {}",
                    "Allowed tools".bold(),
                    cmd.allowed_tools.join(", ")
                );
            }
        }
        cc_schema::ResourceType::Agent => {
            let (agent, origin) = cc_core::agent::load_agent(name, project_dir)?;
            println!("{}: {}", "Agent".bold(), agent.name);
            println!("  {}: {} ({})", "Origin".bold(), origin, origin);
            println!("  {}: {}", "Description".bold(), agent.description);
            if let Some(ref m) = agent.model {
                println!("  {}: {m}", "Model".bold());
            }
            if !agent.tools.is_empty() {
                println!("  {}: {}", "Tools".bold(), agent.tools.join(", "));
            }
        }
        cc_schema::ResourceType::Rule => {
            let (rule, origin) = cc_core::rule::load_rule(name, project_dir)?;
            println!("{}: {}", "Rule".bold(), rule.name);
            println!("  {}: {}", "Origin".bold(), origin);
            if !rule.body.is_empty() {
                println!();
                println!("{}", rule.body);
            }
        }
        cc_schema::ResourceType::Hook => {
            bail!("Use 'cc list hooks' to view hooks.");
        }
    }
    Ok(())
}
