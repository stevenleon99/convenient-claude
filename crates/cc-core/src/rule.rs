use crate::error::CoreError;
use crate::paths::resource_dir;
use cc_schema::{io as schema_io, Origin, ResourceType, Rule};
use std::path::Path;

/// Load a rule by name from the project.
pub fn load_rule(name: &str, project_dir: &Path) -> Result<(Rule, Origin), CoreError> {
    let dir = resource_dir(project_dir, ResourceType::Rule);
    let path = dir.join(format!("{name}.md"));

    if path.exists() {
        let content = schema_io::read_file(&path)?;
        let rule = Rule::parse(&content, &path);
        return Ok((rule, Origin::Project));
    }

    Err(CoreError::ResourceNotFound {
        resource_type: ResourceType::Rule,
        name: name.to_string(),
    })
}

/// Remove a rule by name.
pub fn remove_rule(name: &str, project_dir: &Path) -> Result<(), CoreError> {
    let dir = resource_dir(project_dir, ResourceType::Rule);
    let path = dir.join(format!("{name}.md"));

    if !path.exists() {
        return Err(CoreError::ResourceNotFound {
            resource_type: ResourceType::Rule,
            name: name.to_string(),
        });
    }

    std::fs::remove_file(&path)?;
    Ok(())
}
