use super::ResourceEntry;
use crate::paths::origin_resource_dir;
use crate::workspace::WorkspaceConfig;
use cc_schema::{io as schema_io, Agent, Command, Rule, Skill};
use cc_schema::{Origin, ResourceType};
use std::path::PathBuf;
use std::collections::HashMap;
use std::path::Path;

/// Case-insensitive folder name candidates for each resource type.
fn folder_candidates(resource_type: ResourceType) -> Vec<&'static str> {
    match resource_type {
        ResourceType::Skill => vec!["skills", "skill", "SKILL", "Skills"],
        ResourceType::Command => vec!["commands", "command", "COMMAND", "Commands"],
        ResourceType::Agent => vec!["agents", "agent", "AGENT", "Agents"],
        ResourceType::Rule => vec!["rules", "rule", "RULE", "Rules"],
        ResourceType::Hook => vec!["hooks", "hook", "HOOK", "Hooks"],
        ResourceType::Plugin => vec!["plugins", "plugin", "PLUGIN", "Plugins"],
    }
}

/// File name to look for inside subdirectory-style resources.
/// e.g. skills/angular-architect/SKILL.md
fn subdir_marker(resource_type: ResourceType) -> Option<&'static str> {
    match resource_type {
        ResourceType::Skill => Some("SKILL.md"),
        ResourceType::Command => Some("COMMAND.md"),
        ResourceType::Agent => Some("AGENT.md"),
        ResourceType::Rule => None,
        _ => None,
    }
}

/// Discover all resources of a given type across all registries in cc-workspace.toml.
pub fn discover_resources(
    resource_type: ResourceType,
    project_dir: &Path,
    _extern_libs: &[String],
) -> Vec<ResourceEntry> {
    let mut entries = Vec::new();

    // Find workspace root by searching upward for resource/cc-workspace.toml
    let workspace_root = find_workspace_root(project_dir);

    // Try loading workspace config
    if let Some(config) = WorkspaceConfig::load(&workspace_root) {
        let mut registries = config.registries(&workspace_root);

        // Also include other_plugin directories as registries
        // (e.g., ~/.claude/plugins/marketplaces/fullstack-dev-skills)
        registries.extend(config.other_plugin_registries());

        for registry in &registries {
            let origin = classify_origin(&registry.label);
            discover_from_registry(
                &registry.path,
                resource_type,
                &origin,
                &registry.label,
                &mut entries,
            );
            // Also check .claude/ subdirectory inside external projects
            if matches!(origin, Origin::External { .. }) {
                let claude_sub = registry.path.join(".claude");
                if claude_sub.is_dir() {
                    discover_from_registry(
                        &claude_sub,
                        resource_type,
                        &origin,
                        &registry.label,
                        &mut entries,
                    );
                }
            }
        }
        // Deduplicate by (name, resource_type) — keep first found (external before local/project)
        deduplicate_entries(&mut entries);
    } else {
        // Fallback: use the old origin-based discovery if no workspace config
        let origins = build_fallback_origins(project_dir);
        for origin in &origins {
            let dir = origin_resource_dir(origin, resource_type, project_dir);
            if dir.is_dir() {
                scan_resource_dir(&dir, resource_type, origin, None, &mut entries);
            }
        }
    }

    entries
}

/// Discover all resources across all types.
pub fn discover_all_resources(
    project_dir: &Path,
    extern_libs: &[String],
) -> HashMap<ResourceType, Vec<ResourceEntry>> {
    let mut map = HashMap::new();
    for rt in ResourceType::all() {
        map.insert(*rt, discover_resources(*rt, project_dir, extern_libs));
    }
    map
}

/// Classify a registry label into an Origin.
fn classify_origin(label: &str) -> Origin {
    if label == "project" {
        Origin::Project
    } else if label == "local" {
        Origin::User
    } else {
        Origin::External {
            library: label.to_string(),
        }
    }
}

/// Scan a registry root directory for resources of a given type.
fn discover_from_registry(
    registry_path: &Path,
    resource_type: ResourceType,
    origin: &Origin,
    registry_label: &str,
    entries: &mut Vec<ResourceEntry>,
) {
    let candidates = folder_candidates(resource_type);

    for candidate in &candidates {
        let dir = registry_path.join(candidate);
        if dir.is_dir() {
            scan_resource_dir(&dir, resource_type, origin, Some(registry_label), entries);
        }
    }
}

/// Scan a single resource directory for .md files and subdirectory patterns.
fn scan_resource_dir(
    dir: &Path,
    resource_type: ResourceType,
    origin: &Origin,
    registry: Option<&str>,
    entries: &mut Vec<ResourceEntry>,
) {
    if resource_type == ResourceType::Hook {
        return;
    }

    let marker = subdir_marker(resource_type);

    // Read directory entries
    let Ok(dir_entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in dir_entries.flatten() {
        let path = entry.path();

        // Flat .md files: skills/commit.md
        if path.extension().is_some_and(|ext| ext == "md") {
            if let Some(entry) = load_entry_from_file(&path, resource_type, origin.clone(), registry)
            {
                entries.push(entry);
            }
        }
        // Subdirectory pattern: skills/angular-architect/SKILL.md
        else if path.is_dir() {
            if let Some(marker_name) = marker {
                let marker_path = path.join(marker_name);
                if marker_path.exists() {
                    if let Some(entry) =
                        load_entry_from_file(&marker_path, resource_type, origin.clone(), registry)
                    {
                        entries.push(entry);
                    }
                }
            }
        }
    }
}

/// Load a resource entry from a single .md file.
fn load_entry_from_file(
    path: &Path,
    resource_type: ResourceType,
    origin: Origin,
    registry: Option<&str>,
) -> Option<ResourceEntry> {
    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");

    // If the file is a marker (SKILL.md, COMMAND.md, AGENT.md), use the parent directory as name.
    // Otherwise (flat .md files), use the file stem as the name.
    let marker = subdir_marker(resource_type)
        .map(|m| m.trim_end_matches(".md"))
        .unwrap_or("");
    let name = if file_stem.eq_ignore_ascii_case(marker) {
        path.parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or(file_stem)
            .to_string()
    } else {
        file_stem.to_string()
    };

    let content = schema_io::read_file(path).ok()?;

    let description = match resource_type {
        ResourceType::Skill => Skill::parse(&content, path).ok().map(|s| s.description),
        ResourceType::Command => Command::parse(&content, path).ok().map(|c| c.description),
        ResourceType::Agent => Agent::parse(&content, path).ok().map(|a| a.description),
        ResourceType::Rule => {
            let rule = Rule::parse(&content, path);
            truncate(&rule.body, 80)
        }
        ResourceType::Hook => None,
        ResourceType::Plugin => None,
    };

    Some(ResourceEntry {
        name,
        resource_type,
        origin,
        path: path.to_path_buf(),
        active: false,
        description,
        registry: registry.map(|s| s.to_string()),
    })
}

fn build_fallback_origins(project_dir: &Path) -> Vec<Origin> {
    let mut origins: Vec<Origin> = crate::sync::list_extern_libs(project_dir)
        .into_iter()
        .map(|lib| Origin::External { library: lib })
        .collect();
    origins.push(Origin::User);
    origins.push(Origin::Project);
    origins
}

fn truncate(s: &str, max: usize) -> Option<String> {
    if s.is_empty() {
        return None;
    }
    let first_line = s.lines().next().unwrap_or("");
    if first_line.len() <= max {
        Some(first_line.to_string())
    } else {
        Some(format!("{}...", &first_line[..max]))
    }
}

/// Remove duplicate entries with the same (name, resource_type, registry).
fn deduplicate_entries(entries: &mut Vec<ResourceEntry>) {
    let mut seen = std::collections::HashSet::new();
    entries.retain(|e| {
        let key = (e.name.clone(), e.resource_type, e.registry.clone());
        seen.insert(key)
    });
}

/// Search upward from a directory to find the workspace root (containing resource/cc-workspace.toml).
fn find_workspace_root(start: &Path) -> PathBuf {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join("resource").join("cc-workspace.toml").exists() {
            return dir;
        }
        if !dir.pop() {
            return start.to_path_buf();
        }
    }
}
