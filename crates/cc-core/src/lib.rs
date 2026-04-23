pub mod agent;
pub mod command;
pub mod config;
pub mod error;
pub mod hook;
pub mod init;
pub mod paths;
pub mod resource;
pub mod rule;
pub mod session;
pub mod skill;
pub mod stats;
pub mod sync;
pub mod validate;

// Re-export primary types.
pub use error::CoreError;
pub use init::init_project;
pub use resource::{discover_resources, install_resource, resolve_resources, ResourceEntry};
pub use session::{SessionContext, SessionTracker};
pub use validate::{validate_project, ValidationFinding, ValidationLevel};
