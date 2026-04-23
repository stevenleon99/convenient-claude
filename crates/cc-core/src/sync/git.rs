use std::path::Path;
use std::process::Command;

/// Sync an external library via git submodule update.
pub fn sync_extern(project_dir: &Path, library: &str, dry_run: bool) -> super::SyncResult {
    let submodule_path = format!("extern/{library}");

    if dry_run {
        // Check if there are updates available
        let output = Command::new("git")
            .args(["submodule", "status", &submodule_path])
            .current_dir(project_dir)
            .output();

        match output {
            Ok(out) => {
                let status = String::from_utf8_lossy(&out.stdout);
                let needs_update =
                    status.trim_start().starts_with('+') || status.trim_start().starts_with('-');
                super::SyncResult {
                    library: library.to_string(),
                    updated: false,
                    message: if needs_update {
                        format!("{library}: updates available")
                    } else {
                        format!("{library}: up to date")
                    },
                }
            }
            Err(e) => super::SyncResult {
                library: library.to_string(),
                updated: false,
                message: format!("{library}: failed to check status: {e}"),
            },
        }
    } else {
        // Actually update the submodule
        let output = Command::new("git")
            .args(["submodule", "update", "--remote", &submodule_path])
            .current_dir(project_dir)
            .output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    super::SyncResult {
                        library: library.to_string(),
                        updated: true,
                        message: format!("{library}: synced successfully"),
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    super::SyncResult {
                        library: library.to_string(),
                        updated: false,
                        message: format!("{library}: sync failed: {stderr}"),
                    }
                }
            }
            Err(e) => super::SyncResult {
                library: library.to_string(),
                updated: false,
                message: format!("{library}: git command failed: {e}"),
            },
        }
    }
}
