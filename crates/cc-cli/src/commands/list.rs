use crate::args::{ListTarget, OutputFormat};
use crate::output::{self, Format};
use anyhow::Result;
use cc_schema::ResourceType;
use std::path::Path;

pub fn run(target: &ListTarget, project_dir: &Path, app_dir: &Path) -> Result<()> {
    match target {
        ListTarget::Skills { filter, format } => {
            list_resource(ResourceType::Skill, filter.as_deref(), format, project_dir, app_dir)
        }
        ListTarget::Commands { filter, format } => list_resource(
            ResourceType::Command,
            filter.as_deref(),
            format,
            project_dir,
            app_dir,
        ),
        ListTarget::Agents { filter, format } => {
            list_resource(ResourceType::Agent, filter.as_deref(), format, project_dir, app_dir)
        }
        ListTarget::Hooks { format } => list_hooks(format, project_dir),
        ListTarget::Rules { filter, format } => {
            list_resource(ResourceType::Rule, filter.as_deref(), format, project_dir, app_dir)
        }
        ListTarget::Plugins { format } => list_plugins(format, project_dir, app_dir),
        ListTarget::All { format } => list_all(format, project_dir, app_dir),
    }
}

fn list_resource(
    resource_type: ResourceType,
    filter: Option<&str>,
    format: &OutputFormat,
    project_dir: &Path,
    app_dir: &Path,
) -> Result<()> {
    let mut entries = cc_core::discover_resources(resource_type, project_dir, app_dir);

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

fn list_all(format: &OutputFormat, project_dir: &Path, app_dir: &Path) -> Result<()> {
    let fmt = match format {
        OutputFormat::Table => Format::Table,
        OutputFormat::Json => Format::Json,
        OutputFormat::Plain => Format::Plain,
    };

    for rt in ResourceType::all() {
        if *rt == ResourceType::Hook {
            continue;
        }
        let mut entries = cc_core::discover_resources(*rt, project_dir, app_dir);
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

fn list_plugins(format: &OutputFormat, project_dir: &Path, app_dir: &Path) -> Result<()> {
    use cc_core::ResourceEntry;

    let mut plugins: Vec<(String, std::path::PathBuf, cc_schema::Origin, Option<String>, String)> = Vec::new();

    // Collect candidate directories to scan for plugins
    let mut scan_dirs: Vec<(std::path::PathBuf, String)> = Vec::new();

    // 1. Project-local: .claude/plugins/marketplaces
    let project_plugins = project_dir.join(".claude").join("plugins").join("marketplaces");
    scan_dirs.push((project_plugins, "project".to_string()));

    // 2. User-local: ~/.claude/plugins/marketplaces
    let user_plugins = cc_core::paths::user_claude_dir().join("plugins").join("marketplaces");
    scan_dirs.push((user_plugins, "local".to_string()));

    // 3. External registries from workspace config
    if let Some(config) = cc_core::WorkspaceConfig::load(app_dir) {
        let registries = config.registries(app_dir);
        for registry in &registries {
            // Direct plugins/marketplaces folder
            let dir = registry.path.join("plugins").join("marketplaces");
            if dir.is_dir() {
                scan_dirs.push((dir, registry.label.clone()));
            }
            // .claude/plugins/marketplaces inside external projects
            let claude_plugins = registry.path.join(".claude").join("plugins").join("marketplaces");
            if claude_plugins.is_dir() {
                scan_dirs.push((claude_plugins, registry.label.clone()));
            }
        }
    }

    // Scan each directory for plugin subdirectories
    let mut seen = std::collections::HashSet::new();
    for (dir, registry) in &scan_dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let name = match entry.file_name().to_str() {
                    Some(n) if !n.starts_with('.') => n.to_string(),
                    _ => continue,
                };
                // Deduplicate by name
                if !seen.insert(name.clone()) {
                    continue;
                }

                // Try to read a description from a README.md or PLUGIN.md inside
                let description = read_plugin_description(&path);

                let origin = match registry.as_str() {
                    "project" => cc_schema::Origin::Project,
                    "local" => cc_schema::Origin::User,
                    _ => cc_schema::Origin::External {
                        library: registry.clone(),
                    },
                };

                plugins.push((
                    name,
                    path,
                    origin,
                    description,
                    registry.clone(),
                ));
            }
        }
    }

    if plugins.is_empty() {
        println!("No plugins found.");
        return Ok(());
    }

    match format {
        OutputFormat::Json => {
            let json_entries: Vec<serde_json::Value> = plugins
                .iter()
                .map(|(name, path, origin, desc, registry)| {
                    serde_json::json!({
                        "name": name,
                        "type": "plugin",
                        "registry": registry,
                        "origin": origin.to_string(),
                        "path": path.to_string_lossy(),
                        "description": desc,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_entries)?);
        }
        OutputFormat::Plain => {
            for (name, _, origin, desc, registry) in &plugins {
                let d = desc.as_deref().unwrap_or("");
                if d.is_empty() {
                    println!("{} [plugin/{}] ({})", name, registry, origin);
                } else {
                    println!("{} [plugin/{}] ({})", name, registry, d);
                }
            }
        }
        OutputFormat::Table => {
            use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, ContentArrangement, Table};
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_content_arrangement(ContentArrangement::Dynamic);
            table.set_header(vec!["Name", "Registry", "Origin", "Description"]);
            for (name, _, origin, desc, registry) in &plugins {
                table.add_row(vec![
                    Cell::new(name),
                    Cell::new(registry),
                    Cell::new(origin.to_string()),
                    Cell::new(desc.as_deref().unwrap_or("-")),
                ]);
            }
            println!("{table}");
        }
    }
    Ok(())
}

/// Try to read a short description from a plugin directory.
fn read_plugin_description(dir: &Path) -> Option<String> {
    for candidate in &["PLUGIN.md", "README.md", "plugin.md", "readme.md"] {
        let path = dir.join(candidate);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                // Take the first non-empty, non-heading line
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        continue;
                    }
                    if trimmed.len() > 120 {
                        return Some(format!("{}...", &trimmed[..120]));
                    }
                    return Some(trimmed.to_string());
                }
            }
        }
    }
    None
}
