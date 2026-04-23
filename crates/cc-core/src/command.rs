use crate::error::CoreError;
use crate::paths::origin_resource_dir;
use cc_schema::{io as schema_io, Command, Origin, ResourceType};
use std::path::Path;

/// Load a command by name from all origins, returning the highest-precedence one.
pub fn load_command(name: &str, project_dir: &Path) -> Result<(Command, Origin), CoreError> {
    let origins = [Origin::Project, Origin::User];

    for origin in origins {
        let dir = origin_resource_dir(&origin, ResourceType::Command, project_dir);
        let path = dir.join(format!("{name}.md"));
        if path.exists() {
            let content = schema_io::read_file(&path)?;
            let cmd = Command::parse(&content, &path)?;
            return Ok((cmd, origin));
        }
    }

    Err(CoreError::ResourceNotFound {
        resource_type: ResourceType::Command,
        name: name.to_string(),
    })
}
