use crate::output;
use anyhow::Result;
use std::path::Path;

pub fn run(project_dir: &Path, dry_run: bool) -> Result<()> {
    let libs = cc_core::sync::list_extern_libs(project_dir);

    if libs.is_empty() {
        println!("No external libraries found in extern/.");
        return Ok(());
    }

    if dry_run {
        println!("Checking extern/ updates...\n");
    } else {
        println!("Syncing extern/ libraries...\n");
    }

    for lib in &libs {
        let result = cc_core::sync::sync_extern(project_dir, lib, dry_run);
        if result.updated || dry_run {
            println!("  {}", result.message);
        } else {
            println!("  {} (up to date)", result.message);
        }
    }

    if dry_run {
        println!("\nRun 'cc sync' to apply these changes.");
    } else {
        output::print_success("Sync complete.");
    }

    Ok(())
}
