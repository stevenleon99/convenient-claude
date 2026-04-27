use super::{ValidationFinding, ValidationLevel};
use crate::paths::{claude_dir, resource_dir};
use cc_schema::{io as schema_io, ResourceType};
use std::path::Path;

/// Validate all resources in a project.
pub fn validate_project(project_dir: &Path) -> Vec<ValidationFinding> {
    let mut findings = Vec::new();
    let claude = claude_dir(project_dir);

    // Check .claude directory exists
    if !claude.is_dir() {
        findings.push(ValidationFinding {
            level: ValidationLevel::Error,
            resource_type: None,
            resource_name: None,
            path: Some(claude.clone()),
            message: ".claude/ directory not found".into(),
        });
        return findings;
    }

    // Validate settings.json
    validate_settings(&claude, &mut findings);

    // Validate each resource type
    for rt in &[
        ResourceType::Skill,
        ResourceType::Command,
        ResourceType::Agent,
        ResourceType::Rule,
    ] {
        validate_resources(project_dir, *rt, &mut findings);
    }

    findings
}

fn validate_settings(claude_dir: &Path, findings: &mut Vec<ValidationFinding>) {
    let path = claude_dir.join("settings.json");
    if !path.exists() {
        findings.push(ValidationFinding {
            level: ValidationLevel::Warning,
            resource_type: None,
            resource_name: None,
            path: Some(path),
            message: "settings.json not found (will use defaults)".into(),
        });
        return;
    }

    match schema_io::read_json::<cc_schema::Settings>(&path) {
        Ok(settings) => {
            // Check for duplicate permission entries
            let mut seen = std::collections::HashSet::new();
            for entry in &settings.permissions.allow {
                if !seen.insert(entry) {
                    findings.push(ValidationFinding {
                        level: ValidationLevel::Warning,
                        resource_type: None,
                        resource_name: None,
                        path: Some(path.clone()),
                        message: format!("duplicate permission entry: '{entry}'"),
                    });
                }
            }
        }
        Err(e) => {
            findings.push(ValidationFinding {
                level: ValidationLevel::Error,
                resource_type: None,
                resource_name: None,
                path: Some(path),
                message: format!("invalid settings.json: {e}"),
            });
        }
    }
}

fn validate_resources(
    project_dir: &Path,
    resource_type: ResourceType,
    findings: &mut Vec<ValidationFinding>,
) {
    let dir = resource_dir(project_dir, resource_type);
    if !dir.is_dir() {
        return;
    }

    let files = schema_io::list_md_files(&dir).unwrap_or_default();
    for path in files {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        if let Err(e) = validate_resource_file(&path, resource_type) {
            findings.push(ValidationFinding {
                level: ValidationLevel::Error,
                resource_type: Some(resource_type),
                resource_name: Some(name.to_string()),
                path: Some(path),
                message: e,
            });
        }
    }
}

fn validate_resource_file(path: &Path, resource_type: ResourceType) -> Result<(), String> {
    let content = schema_io::read_file(path).map_err(|e| e.to_string())?;

    match resource_type {
        ResourceType::Skill => {
            cc_schema::Skill::parse(&content, path).map_err(|e| e.to_string())?;
        }
        ResourceType::Command => {
            cc_schema::Command::parse(&content, path).map_err(|e| e.to_string())?;
        }
        ResourceType::Agent => {
            cc_schema::Agent::parse(&content, path).map_err(|e| e.to_string())?;
        }
        ResourceType::Rule => {
            // Rules are plain markdown, always valid
        }
        ResourceType::Hook => {
            // Hooks validated in settings validation
        }
        ResourceType::Plugin => {
            // Plugins are read-only, no validation needed
        }
    }

    Ok(())
}
