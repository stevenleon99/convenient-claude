use crate::error::CoreError;
use crate::paths::origin_resource_dir;
use cc_schema::{io as schema_io, Agent, Origin, ResourceType};
use std::path::Path;

/// Load an agent by name from all origins, returning the highest-precedence one.
pub fn load_agent(name: &str, project_dir: &Path) -> Result<(Agent, Origin), CoreError> {
    let origins = [Origin::Project, Origin::User];

    for origin in origins {
        let dir = origin_resource_dir(&origin, ResourceType::Agent, project_dir);
        let path = dir.join(format!("{name}.md"));
        if path.exists() {
            let content = schema_io::read_file(&path)?;
            let agent = Agent::parse(&content, &path)?;
            return Ok((agent, origin));
        }
    }

    Err(CoreError::ResourceNotFound {
        resource_type: ResourceType::Agent,
        name: name.to_string(),
    })
}
