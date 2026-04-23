mod git;

pub use git::sync_extern;

use std::path::Path;

/// Result of syncing external libraries.
#[derive(Debug)]
pub struct SyncResult {
    pub library: String,
    pub updated: bool,
    pub message: String,
}

/// Get list of external library names from the `extern/` directory.
pub fn list_extern_libs(project_dir: &Path) -> Vec<String> {
    let extern_dir = project_dir.join("extern");
    if !extern_dir.is_dir() {
        return Vec::new();
    }

    let mut libs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&extern_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    // Skip hidden directories
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
