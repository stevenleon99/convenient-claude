use super::merge::merge_settings;
use crate::error::CoreError;
use crate::hook;
use crate::paths::{claude_dir, user_claude_dir};
use cc_schema::{io as schema_io, Origin, Settings};

/// Load the project's settings.json.
pub fn load_project_settings(project_dir: &std::path::Path) -> Result<Settings, CoreError> {
    hook::load_settings(project_dir)
}

/// Load the user's settings.json.
pub fn load_user_settings() -> Result<Settings, CoreError> {
    hook::load_user_settings()
}

/// Load the effective (merged) settings.
pub fn load_effective_settings(
    project_dir: &std::path::Path,
) -> Result<super::EffectiveConfig, CoreError> {
    let user = load_user_settings()?;
    let project = load_project_settings(project_dir)?;
    let merged = merge_settings(&user, &project);

    Ok(super::EffectiveConfig {
        user,
        project,
        merged,
    })
}

/// Set a config value in the project settings.
/// Supports dot-notation keys like `permissions.allow`.
pub fn set_config_value(
    project_dir: &std::path::Path,
    key: &str,
    value: &str,
    scope: &Origin,
) -> Result<(), CoreError> {
    let base_dir = match scope {
        Origin::Project => claude_dir(project_dir),
        Origin::User => user_claude_dir(),
        _ => {
            return Err(CoreError::ValidationFailed {
                details: "Can only set config in project or user scope".into(),
            });
        }
    };

    let path = base_dir.join("settings.json");
    let mut settings: Settings = if path.exists() {
        schema_io::read_json(&path)?
    } else {
        Settings::default()
    };

    match key {
        "permissions.allow" => {
            if !settings.permissions.allow.contains(&value.to_string()) {
                settings.permissions.allow.push(value.to_string());
            }
        }
        "permissions.deny" => {
            if !settings.permissions.deny.contains(&value.to_string()) {
                settings.permissions.deny.push(value.to_string());
            }
        }
        _ => {
            return Err(CoreError::ValidationFailed {
                details: format!("Unknown config key: {key}"),
            });
        }
    }

    std::fs::create_dir_all(&base_dir)?;
    schema_io::write_json(&path, &settings)?;
    Ok(())
}
