mod discovery;
mod install;
mod resolution;

pub use discovery::discover_resources;
pub use install::install_resource;
pub use resolution::resolve_resources;

use cc_schema::{Origin, ResourceType};
use std::path::PathBuf;

/// A resource entry discovered from a specific origin.
#[derive(Debug, Clone)]
pub struct ResourceEntry {
    /// The name of the resource.
    pub name: String,
    /// The type of resource.
    pub resource_type: ResourceType,
    /// Where this resource was found.
    pub origin: Origin,
    /// The filesystem path to the resource file.
    pub path: PathBuf,
    /// Whether this is the "active" version (highest precedence).
    pub active: bool,
    /// A short description (if available).
    pub description: Option<String>,
    /// Which registry in cc-workspace.toml this came from (e.g. "extern/claude-skills", "local", "project").
    pub registry: Option<String>,
}
