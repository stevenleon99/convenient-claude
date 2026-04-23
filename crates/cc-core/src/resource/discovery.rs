use super::ResourceEntry;
use crate::paths::origin_resource_dir;
use cc_schema::{io as schema_io, Agent, Command, Rule, Skill};
use cc_schema::{Origin, ResourceType};
use std::collections::HashMap;
use std::path::Path;

/// Discover all resources of a given type across all origins.
pub fn discover_resources(
    resource_type: ResourceType,
    project_dir: &Path,
    extern_libs: &[String],
) -> Vec<ResourceEntry> {
    let origins = build_origins(project_dir, extern_libs);
    let mut entries = Vec::new();

    for origin in &origins {
        let dir = origin_resource_dir(origin, resource_type, project_dir);
        if !dir.is_dir() {
            continue;
        }
        match resource_type {
            ResourceType::Skill => {
                if let Ok(files) = schema_io::list_md_files(&dir) {
                    for path in files {
                        if let Some(entry) = load_skill_entry(&path, origin.clone()) {
                            entries.push(entry);
                        }
                    }
                }
            }
            ResourceType::Command => {
                if let Ok(files) = schema_io::list_md_files(&dir) {
                    for path in files {
                        if let Some(entry) = load_command_entry(&path, origin.clone()) {
                            entries.push(entry);
                        }
                    }
                }
            }
            ResourceType::Agent => {
                if let Ok(files) = schema_io::list_md_files(&dir) {
                    for path in files {
                        if let Some(entry) = load_agent_entry(&path, origin.clone()) {
                            entries.push(entry);
                        }
                    }
                }
            }
            ResourceType::Rule => {
                if let Ok(files) = schema_io::list_md_files(&dir) {
                    for path in files {
                        let rule =
                            Rule::parse(&schema_io::read_file(&path).unwrap_or_default(), &path);
                        entries.push(ResourceEntry {
                            name: rule.name,
                            resource_type: ResourceType::Rule,
                            origin: origin.clone(),
                            path,
                            active: false,
                            description: truncate(&rule.body, 80),
                        });
                    }
                }
            }
            ResourceType::Hook => {
                // Hooks are discovered via settings, not file scanning
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

fn build_origins(_project_dir: &Path, extern_libs: &[String]) -> Vec<Origin> {
    let mut origins: Vec<Origin> = extern_libs
        .iter()
        .map(|lib| Origin::External {
            library: lib.clone(),
        })
        .collect();
    origins.push(Origin::User);
    origins.push(Origin::Project);
    origins
}

fn load_skill_entry(path: &Path, origin: Origin) -> Option<ResourceEntry> {
    let content = schema_io::read_file(path).ok()?;
    let skill = Skill::parse(&content, path).ok()?;
    let name = path.file_stem()?.to_str()?.to_string();
    Some(ResourceEntry {
        name,
        resource_type: ResourceType::Skill,
        origin,
        path: path.to_path_buf(),
        active: false,
        description: Some(skill.description),
    })
}

fn load_command_entry(path: &Path, origin: Origin) -> Option<ResourceEntry> {
    let content = schema_io::read_file(path).ok()?;
    let cmd = Command::parse(&content, path).ok()?;
    let name = path.file_stem()?.to_str()?.to_string();
    Some(ResourceEntry {
        name,
        resource_type: ResourceType::Command,
        origin,
        path: path.to_path_buf(),
        active: false,
        description: Some(cmd.description),
    })
}

fn load_agent_entry(path: &Path, origin: Origin) -> Option<ResourceEntry> {
    let content = schema_io::read_file(path).ok()?;
    let agent = Agent::parse(&content, path).ok()?;
    let name = path.file_stem()?.to_str()?.to_string();
    Some(ResourceEntry {
        name,
        resource_type: ResourceType::Agent,
        origin,
        path: path.to_path_buf(),
        active: false,
        description: Some(agent.description),
    })
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
