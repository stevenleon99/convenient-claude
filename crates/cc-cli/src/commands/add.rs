use crate::args::AddTarget;
use crate::output;
use anyhow::{bail, Result};
use cc_schema::{Origin, ResourceType};
use std::path::Path;

pub fn run(target: &AddTarget, project_dir: &Path, workspace_root: &Path) -> Result<()> {
    match target {
        AddTarget::Skill {
            name,
            from,
            to,
            force,
        } => add_resource(ResourceType::Skill, name, from, to, *force, project_dir, workspace_root),
        AddTarget::Command {
            name,
            from,
            to,
            force,
        } => add_resource(ResourceType::Command, name, from, to, *force, project_dir, workspace_root),
        AddTarget::Agent {
            name,
            from,
            to,
            force,
        } => add_resource(ResourceType::Agent, name, from, to, *force, project_dir, workspace_root),
        AddTarget::Hook {
            event,
            cmd,
            matcher,
        } => add_hook(event, cmd, matcher.as_deref(), project_dir),
        AddTarget::Rule { name, from, force } => add_rule(name, from, *force, project_dir),
    }
}

fn add_resource(
    resource_type: ResourceType,
    name: &str,
    from: &Option<String>,
    to: &str,
    force: bool,
    project_dir: &Path,
    workspace_root: &Path,
) -> Result<()> {
    let target_origin = match to {
        "user" => Origin::User,
        _ => Origin::Project,
    };

    // Discover from all origins or specific source
    let extern_libs = cc_core::list_extern_libs(project_dir);
    let sources = if let Some(source) = from {
        if source.starts_with("extern/") {
            vec![source.trim_start_matches("extern/").to_string()]
        } else {
            extern_libs
        }
    } else {
        extern_libs
    };

    let mut entries =
        cc_core::discover_resources(resource_type, workspace_root, &sources);
    cc_core::resolve_resources(&mut entries);

    // Find the requested resource
    let source = entries
        .iter()
        .find(|e| e.name == name)
        .ok_or_else(|| anyhow::anyhow!(
            "{} '{}' not found in any registry.\n\nRun 'cc list {}s' to see available resources.",
            resource_type, name, resource_type
        ))?;

    // Determine the source directory to copy (the folder, not just the .md file)
    let source_dir = source.path.parent().unwrap_or_else(|| Path::new("."));
    let dest_dir = cc_core::paths::origin_resource_dir(&target_origin, resource_type, project_dir)
        .join(&source.name);

    if dest_dir.exists() && !force {
        bail!(
            "{} '{}' already exists at {}. Use --force to overwrite.",
            resource_type,
            name,
            dest_dir.display()
        );
    }

    // Remove existing if force
    if dest_dir.exists() && force {
        std::fs::remove_dir_all(&dest_dir)?;
    }

    // Copy the entire folder
    std::fs::create_dir_all(&dest_dir)?;
    copy_dir_recursive(source_dir, &dest_dir)?;

    output::print_success(&format!(
        "Installed {} \"{}\" → {}",
        resource_type,
        source.name,
        dest_dir.display()
    ));

    Ok(())
}

/// Recursively copy a directory's contents into the destination.
fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            std::fs::create_dir_all(&dest_path)?;
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path)?;
        }
    }
    Ok(())
}

fn add_hook(event: &str, cmd: &str, matcher: Option<&str>, project_dir: &Path) -> Result<()> {
    let hook_event = parse_hook_event(event)?;
    let hook_entry = cc_schema::HookEntry {
        hook_type: "command".to_string(),
        command: cmd.to_string(),
    };

    cc_core::hook::add_hook(
        project_dir,
        hook_event,
        matcher.map(String::from),
        hook_entry,
    )?;

    output::print_success(&format!(
        "Added hook on {event} → {cmd}{}",
        matcher
            .map(|m| format!(" (matcher: {m})"))
            .unwrap_or_default()
    ));
    Ok(())
}

fn add_rule(name: &str, from: &Option<String>, force: bool, project_dir: &Path) -> Result<()> {
    // Rules are simpler - just create a placeholder
    let dir = project_dir.join(".claude").join("rules");
    std::fs::create_dir_all(&dir)?;

    let path = dir.join(format!("{name}.md"));
    if path.exists() && !force {
        bail!("Rule '{name}' already exists. Use --force to overwrite.");
    }

    if let Some(source) = from {
        let source_path = std::path::PathBuf::from(source);
        if source_path.exists() {
            std::fs::copy(&source_path, &path)?;
            output::print_success(&format!("Installed rule \"{name}\" → {}", path.display()));
            return Ok(());
        }
    }

    // Create a minimal rule file
    std::fs::write(&path, format!("# {name}\n\nRule content here.\n"))?;
    output::print_success(&format!("Created rule \"{name}\" → {}", path.display()));
    Ok(())
}

fn parse_hook_event(s: &str) -> Result<cc_schema::HookEvent> {
    match s.to_lowercase().as_str() {
        "pretooluse" | "pre-tool-use" => Ok(cc_schema::HookEvent::PreToolUse),
        "posttooluse" | "post-tool-use" => Ok(cc_schema::HookEvent::PostToolUse),
        "notification" => Ok(cc_schema::HookEvent::Notification),
        "stop" => Ok(cc_schema::HookEvent::Stop),
        _ => bail!(
            "Unknown hook event: '{s}'. Expected: PreToolUse, PostToolUse, Notification, Stop"
        ),
    }
}
