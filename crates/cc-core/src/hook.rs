use crate::error::CoreError;
use crate::paths::{claude_dir, user_claude_dir};
use cc_schema::{io as schema_io, HookConfig, HookEntry, HookEvent, HookMatcher, Settings};
use std::collections::HashMap;
use std::path::Path;

/// Load hooks from the project's settings.json.
pub fn load_hooks(project_dir: &Path) -> Result<HashMap<HookEvent, Vec<HookMatcher>>, CoreError> {
    let settings = load_settings(project_dir)?;
    Ok(settings.hooks.map(|hc| hc.hooks).unwrap_or_default())
}

/// Add a hook entry for a given event.
pub fn add_hook(
    project_dir: &Path,
    event: HookEvent,
    matcher: Option<String>,
    hook: HookEntry,
) -> Result<(), CoreError> {
    let mut settings = load_settings(project_dir)?;
    let hooks = settings.hooks.get_or_insert_with(HookConfig::default);

    let matchers = hooks.hooks.entry(event).or_default();

    // Find existing matcher or create new one
    if let Some(matcher_pattern) = &matcher {
        if let Some(existing) = matchers
            .iter_mut()
            .find(|m| m.matcher.as_deref() == Some(matcher_pattern.as_str()))
        {
            existing.hooks.push(hook);
        } else {
            matchers.push(HookMatcher {
                matcher,
                hooks: vec![hook],
            });
        }
    } else {
        // No matcher — add to a catch-all entry
        matchers.push(HookMatcher {
            matcher: None,
            hooks: vec![hook],
        });
    }

    save_settings(project_dir, &settings)
}

/// Remove a hook by event and command.
pub fn remove_hook(project_dir: &Path, event: HookEvent, command: &str) -> Result<bool, CoreError> {
    let mut settings = load_settings(project_dir)?;
    let removed = if let Some(hooks) = &mut settings.hooks {
        if let Some(matchers) = hooks.hooks.get_mut(&event) {
            let mut found = false;
            for matcher in matchers.iter_mut() {
                matcher.hooks.retain(|h| h.command != command);
                found = true;
            }
            // Clean up empty matchers
            matchers.retain(|m| !m.hooks.is_empty());
            found
        } else {
            false
        }
    } else {
        false
    };

    if removed {
        save_settings(project_dir, &settings)?;
    }
    Ok(removed)
}

/// Load project settings.
pub fn load_settings(project_dir: &Path) -> Result<Settings, CoreError> {
    let path = claude_dir(project_dir).join("settings.json");
    if !path.exists() {
        return Ok(Settings::default());
    }
    let settings = schema_io::read_json(&path)?;
    Ok(settings)
}

/// Save project settings.
pub fn save_settings(project_dir: &Path, settings: &Settings) -> Result<(), CoreError> {
    let path = claude_dir(project_dir).join("settings.json");
    schema_io::write_json(&path, settings)?;
    Ok(())
}

/// Load user settings.
pub fn load_user_settings() -> Result<Settings, CoreError> {
    let path = user_claude_dir().join("settings.json");
    if !path.exists() {
        return Ok(Settings::default());
    }
    let settings = schema_io::read_json(&path)?;
    Ok(settings)
}
