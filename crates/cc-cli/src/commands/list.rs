use crate::args::{ListTarget, OutputFormat};
use crate::output::{self, Format};
use anyhow::Result;
use cc_schema::ResourceType;
use std::path::Path;

pub fn run(target: &ListTarget, project_dir: &Path) -> Result<()> {
    match target {
        ListTarget::Skills { filter, format } => {
            list_resource(ResourceType::Skill, filter.as_deref(), format, project_dir)
        }
        ListTarget::Commands { filter, format } => list_resource(
            ResourceType::Command,
            filter.as_deref(),
            format,
            project_dir,
        ),
        ListTarget::Agents { filter, format } => {
            list_resource(ResourceType::Agent, filter.as_deref(), format, project_dir)
        }
        ListTarget::Hooks { format } => list_hooks(format, project_dir),
        ListTarget::Rules { filter, format } => {
            list_resource(ResourceType::Rule, filter.as_deref(), format, project_dir)
        }
        ListTarget::All { format } => list_all(format, project_dir),
    }
}

fn list_resource(
    resource_type: ResourceType,
    filter: Option<&str>,
    format: &OutputFormat,
    project_dir: &Path,
) -> Result<()> {
    let extern_libs = cc_core::sync::list_extern_libs(project_dir);
    let mut entries = cc_core::discover_resources(resource_type, project_dir, &extern_libs);

    // Apply filter
    if let Some(f) = filter {
        let lower = f.to_lowercase();
        entries.retain(|e| {
            e.name.to_lowercase().contains(&lower)
                || e.description
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&lower)
        });
    }

    cc_core::resolve_resources(&mut entries);
    let fmt = match format {
        OutputFormat::Table => Format::Table,
        OutputFormat::Json => Format::Json,
        OutputFormat::Plain => Format::Plain,
    };
    output::print_resource_list(&entries, fmt);
    Ok(())
}

fn list_hooks(format: &OutputFormat, project_dir: &Path) -> Result<()> {
    use cc_core::hook;

    let hooks = hook::load_hooks(project_dir)?;
    if hooks.is_empty() {
        println!("No hooks configured.");
        return Ok(());
    }

    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&hooks)?;
            println!("{json}");
        }
        _ => {
            for (event, matchers) in &hooks {
                println!("{}:", event_variant_name(event));
                for matcher in matchers {
                    let pattern = matcher.matcher.as_deref().unwrap_or("*");
                    for hook in &matcher.hooks {
                        println!("  [{pattern}] {} ({})", hook.command, hook.hook_type);
                    }
                }
            }
        }
    }
    Ok(())
}

fn list_all(format: &OutputFormat, project_dir: &Path) -> Result<()> {
    let fmt = match format {
        OutputFormat::Table => Format::Table,
        OutputFormat::Json => Format::Json,
        OutputFormat::Plain => Format::Plain,
    };

    for rt in ResourceType::all() {
        if *rt == ResourceType::Hook {
            continue;
        }
        let extern_libs = cc_core::sync::list_extern_libs(project_dir);
        let mut entries = cc_core::discover_resources(*rt, project_dir, &extern_libs);
        cc_core::resolve_resources(&mut entries);
        if !entries.is_empty() {
            println!("\n{}:", rt);
            output::print_resource_list(&entries, fmt);
        }
    }
    Ok(())
}

fn event_variant_name(event: &cc_schema::HookEvent) -> &'static str {
    match event {
        cc_schema::HookEvent::PreToolUse => "PreToolUse",
        cc_schema::HookEvent::PostToolUse => "PostToolUse",
        cc_schema::HookEvent::Notification => "Notification",
        cc_schema::HookEvent::Stop => "Stop",
    }
}
