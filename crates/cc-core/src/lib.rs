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
pub mod validate;
pub mod workspace;

// Re-export primary types.
pub use error::CoreError;
pub use init::{init_project, reinit_project, force_init_project, InitResult, ExistingClaudeDir};
pub use resource::{discover_resources, install_resource, resolve_resources, ResourceEntry};
pub use session::{SessionContext, SessionTracker};
pub use validate::{validate_project, ValidationFinding, ValidationLevel};
pub use workspace::WorkspaceConfig;

/// Get list of external library names from the `extern/` directory.
pub fn list_extern_libs(project_dir: &std::path::Path) -> Vec<String> {
    let extern_dir = project_dir.join("extern");
    if !extern_dir.is_dir() {
        return Vec::new();
    }

    let mut libs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&extern_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    if !name.starts_with('.') {
                        libs.push(name.to_string());
                    }
                }
            }
        }
    }

    libs.sort();
    libs
}
