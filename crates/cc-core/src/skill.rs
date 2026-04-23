use crate::error::CoreError;
use crate::paths::origin_resource_dir;
use cc_schema::{io as schema_io, Origin, ResourceType, Skill};
use std::path::Path;

/// Load a skill by name from all origins, returning the highest-precedence one.
pub fn load_skill(name: &str, project_dir: &Path) -> Result<(Skill, Origin), CoreError> {
    let origins = [Origin::Project, Origin::User];

    for origin in origins {
        let dir = origin_resource_dir(&origin, ResourceType::Skill, project_dir);
        let path = dir.join(format!("{name}.md"));
        if path.exists() {
            let content = schema_io::read_file(&path)?;
            let skill = Skill::parse(&content, &path)?;
            return Ok((skill, origin));
        }
    }

    Err(CoreError::ResourceNotFound {
        resource_type: ResourceType::Skill,
        name: name.to_string(),
    })
}

/// Load a skill from a specific origin.
pub fn load_skill_from_origin(
    name: &str,
    origin: &Origin,
    project_dir: &Path,
) -> Result<Skill, CoreError> {
    let dir = origin_resource_dir(origin, ResourceType::Skill, project_dir);
    let path = dir.join(format!("{name}.md"));
    let content = schema_io::read_file(&path)?;
    let skill = Skill::parse(&content, &path)?;
    Ok(skill)
}
