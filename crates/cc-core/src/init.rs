use crate::error::CoreError;
use crate::paths::claude_dir;
use crate::workspace::WorkspaceConfig;
use cc_schema::Settings;
use std::path::Path;

/// Initialize a `.claude/` directory in the project and register it in cc-workspace.toml.
pub fn init_project(project_dir: &Path, workspace_root: &Path) -> Result<Vec<String>, CoreError> {
    let claude = claude_dir(project_dir);

    if claude.exists() {
        return Err(CoreError::ValidationFailed {
            details: format!(
                ".claude/ directory already exists in {}",
                project_dir.display()
            ),
        });
    }

    let mut created = Vec::new();

    // Create .claude/ root
    std::fs::create_dir_all(&claude)?;
    created.push(".claude/".to_string());

    // Create settings.json with empty permissions
    let settings = Settings::default();
    let settings_path = claude.join("settings.json");
    cc_schema::io::write_json(&settings_path, &settings)?;
    created.push(".claude/settings.json".to_string());

    // Create resource directories
    let dirs = ["skills", "commands", "agents", "rules"];
    for dir in &dirs {
        let path = claude.join(dir);
        std::fs::create_dir_all(&path)?;
        created.push(format!(".claude/{dir}/"));
    }

    // Update cc-workspace.toml with the project path
    WorkspaceConfig::set_current_project(workspace_root, project_dir)?;

    Ok(created)
}
