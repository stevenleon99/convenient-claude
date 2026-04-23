use cc_schema::{Origin, ResourceType, SchemaError};
use std::path::PathBuf;

/// Errors originating from the service/business-logic layer.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("resource not found: {resource_type} '{name}'")]
    ResourceNotFound {
        resource_type: ResourceType,
        name: String,
    },

    #[error(
        "conflict: {resource_type} '{name}' exists in {existing_origin}, cannot install from {new_origin}"
    )]
    Conflict {
        resource_type: ResourceType,
        name: String,
        existing_origin: Origin,
        new_origin: Origin,
    },

    #[error("validation failed: {details}")]
    ValidationFailed { details: String },

    #[error("session already active")]
    SessionActive,

    #[error("no active session")]
    NoSession,

    #[error("project not initialized: no .claude/ directory found in {path}")]
    NotInitialized { path: PathBuf },

    #[error(transparent)]
    Schema(#[from] SchemaError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
