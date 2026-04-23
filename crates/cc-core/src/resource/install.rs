use super::ResourceEntry;
use crate::error::CoreError;
use crate::paths::{origin_resource_dir, resource_dir};
use cc_schema::{io as schema_io, Agent, Command, Rule, Skill};
use cc_schema::{Origin, ResourceType};
use std::path::Path;

/// Result of installing a resource.
#[derive(Debug)]
pub struct InstallResult {
    pub destination: std::path::PathBuf,
    pub resource_type: ResourceType,
    pub name: String,
}

/// Install a resource by copying it from its source origin to the target scope.
pub fn install_resource(
    source: &ResourceEntry,
    target_origin: &Origin,
    project_dir: &Path,
    force: bool,
) -> Result<InstallResult, CoreError> {
    let target_dir = origin_resource_dir(target_origin, source.resource_type, project_dir);
    let extension = source.resource_type.extension();
    let dest_path = target_dir.join(format!("{}.{}", source.name, extension));

    // Check for conflicts
    if dest_path.exists() && !force {
        return Err(CoreError::Conflict {
            resource_type: source.resource_type,
            name: source.name.clone(),
            existing_origin: target_origin.clone(),
            new_origin: source.origin.clone(),
        });
    }

    // Ensure target directory exists
    std::fs::create_dir_all(&target_dir)?;

    // Copy the resource
    let content = schema_io::read_file(&source.path)?;
    schema_io::write_file(&dest_path, &content)?;

    Ok(InstallResult {
        destination: dest_path,
        resource_type: source.resource_type,
        name: source.name.clone(),
    })
}

/// Install a resource from a specific file path.
pub fn install_from_path(
    source_path: &Path,
    resource_type: ResourceType,
    target_origin: &Origin,
    project_dir: &Path,
    force: bool,
) -> Result<InstallResult, CoreError> {
    let content = schema_io::read_file(source_path)?;

    // Validate the resource by parsing it
    let name = match resource_type {
        ResourceType::Skill => {
            let skill = Skill::parse(&content, source_path)?;
            skill.name
        }
        ResourceType::Command => {
            let cmd = Command::parse(&content, source_path)?;
            cmd.name
        }
        ResourceType::Agent => {
            let agent = Agent::parse(&content, source_path)?;
            agent.name
        }
        ResourceType::Rule => {
            let rule = Rule::parse(&content, source_path);
            rule.name
        }
        ResourceType::Hook => {
            return Err(CoreError::ValidationFailed {
                details: "Hooks should be managed via settings, not installed from files".into(),
            });
        }
    };

    let target_dir = resource_dir(project_dir, resource_type);
    let extension = resource_type.extension();
    let dest_path = target_dir.join(format!("{}.{}", name, extension));

    if dest_path.exists() && !force {
        return Err(CoreError::Conflict {
            resource_type,
            name: name.clone(),
            existing_origin: target_origin.clone(),
            new_origin: Origin::External {
                library: "file".into(),
            },
        });
    }

    std::fs::create_dir_all(&target_dir)?;
    schema_io::write_file(&dest_path, &content)?;

    Ok(InstallResult {
        destination: dest_path,
        resource_type,
        name,
    })
}

/// Remove a resource from a given scope.
pub fn remove_resource(
    name: &str,
    resource_type: ResourceType,
    origin: &Origin,
    project_dir: &Path,
) -> Result<(), CoreError> {
    let dir = origin_resource_dir(origin, resource_type, project_dir);
    let extension = resource_type.extension();
    let path = dir.join(format!("{}.{}", name, extension));

    if !path.exists() {
        return Err(CoreError::ResourceNotFound {
            resource_type,
            name: name.to_string(),
        });
    }

    std::fs::remove_file(&path)?;
    Ok(())
}
