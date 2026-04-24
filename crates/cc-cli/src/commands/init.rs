use crate::output;
use anyhow::{bail, Result};
use std::path::Path;

pub fn run(project_dir: &Path, workspace_root: &Path) -> Result<()> {
    match cc_core::init_project(project_dir, workspace_root) {
        Ok(created) => {
            output::print_success(&format!(
                "Initialized .claude/ configuration in {}",
                project_dir.display()
            ));
            println!();
            println!("Created:");
            for item in &created {
                println!("  {item}");
            }
            println!();
            output::print_info(&format!(
                "Project registered in cc-workspace.toml"
            ));
            println!();
            println!("Next steps:");
            println!("  cc list skills              # browse available skills");
            println!("  cc add skill <name>         # install what you need");
        }
        Err(e) => bail!("Init failed: {e}"),
    }
    Ok(())
}
