use crate::args::RemoveTarget;
use crate::output;
use anyhow::{bail, Result};
use cc_schema::{Origin, ResourceType};
use std::path::Path;

pub fn run(target: &RemoveTarget, project_dir: &Path) -> Result<()> {
    match target {
        RemoveTarget::Skill { name } => remove_resource(ResourceType::Skill, name, project_dir),
        RemoveTarget::Command { name } => remove_resource(ResourceType::Command, name, project_dir),
        RemoveTarget::Agent { name } => remove_resource(ResourceType::Agent, name, project_dir),
        RemoveTarget::Hook { event, cmd } => remove_hook(event, cmd, project_dir),
        RemoveTarget::Rule { name } => remove_rule(name, project_dir),
    }
}

fn remove_resource(resource_type: ResourceType, name: &str, project_dir: &Path) -> Result<()> {
    // Try removing from project first, then user
    for origin in &[Origin::Project, Origin::User] {
        let dir = origin_resource_dir(origin, resource_type, project_dir);
        let path = dir.join(format!("{name}.md"));

        if path.exists() {
            std::fs::remove_file(&path)?;
            output::print_success(&format!(
                "Removed {} '{}' from {}",
                resource_type, name, origin
            ));
            return Ok(());
        }
    }

    bail!(
        "{} '{}' not found in any editable origin.",
        resource_type,
        name
    )
}

fn remove_hook(event: &str, cmd: &str, project_dir: &Path) -> Result<()> {
    let hook_event = parse_hook_event(event)?;
    let removed = cc_core::hook::remove_hook(project_dir, hook_event, cmd)?;

    if removed {
        output::print_success(&format!("Removed hook on {event} → {cmd}"));
    } else {
        bail!("Hook '{cmd}' on {event} not found.");
    }
    Ok(())
}

fn remove_rule(name: &str, project_dir: &Path) -> Result<()> {
    cc_core::rule::remove_rule(name, project_dir)?;
    output::print_success(&format!("Removed rule '{name}'"));
    Ok(())
}

fn parse_hook_event(s: &str) -> Result<cc_schema::HookEvent> {
    match s.to_lowercase().as_str() {
        "pretooluse" | "pre-tool-use" => Ok(cc_schema::HookEvent::PreToolUse),
        "posttooluse" | "post-tool-use" => Ok(cc_schema::HookEvent::PostToolUse),
        "notification" => Ok(cc_schema::HookEvent::Notification),
        "stop" => Ok(cc_schema::HookEvent::Stop),
        _ => bail!("Unknown hook event: '{s}'"),
    }
}

fn origin_resource_dir(
    origin: &Origin,
    rt: ResourceType,
    project_dir: &Path,
) -> std::path::PathBuf {
    let base = match origin {
        Origin::Project => project_dir.join(".claude"),
        Origin::User => cc_core::paths::user_claude_dir(),
        _ => project_dir.join(".claude"),
    };
    let dir = rt.dir_name();
    if dir.is_empty() {
        base
    } else {
        base.join(dir)
    }
}
