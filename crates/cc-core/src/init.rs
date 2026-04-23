use crate::error::CoreError;
use crate::paths::claude_dir;
use cc_schema::Settings;
use std::path::Path;

/// Initialize a `.claude/` directory in the project.
pub fn init_project(project_dir: &Path) -> Result<Vec<String>, CoreError> {
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

    Ok(created)
}
