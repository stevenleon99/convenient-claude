mod conflicts;
mod schema;

pub use conflicts::check_conflicts;
pub use schema::validate_project;

use cc_schema::ResourceType;
use std::path::PathBuf;

/// A validation finding (error or warning).
#[derive(Debug)]
pub struct ValidationFinding {
    pub level: ValidationLevel,
    pub resource_type: Option<ResourceType>,
    pub resource_name: Option<String>,
    pub path: Option<PathBuf>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationLevel {
    Error,
    Warning,
}
