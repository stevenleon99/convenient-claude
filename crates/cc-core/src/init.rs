use crate::error::CoreError;
use crate::paths::claude_dir;
use crate::workspace::WorkspaceConfig;
use cc_schema::hook::{HookConfig, HookEntry, HookEvent, HookMatcher};
use cc_schema::Settings;
use std::collections::HashMap;
use std::path::Path;

/// Initialize a `.claude/` directory in the project and register it in cc-workspace.toml.
pub fn init_project(project_dir: &Path, workspace_root: &Path) -> Result<InitResult, CoreError> {
    let claude = claude_dir(project_dir);

    if claude.exists() {
        let existing = scan_existing(&claude);
        return Ok(InitResult::AlreadyExists { existing });
    }

    let created = create_fresh(&claude)?;
    WorkspaceConfig::set_current_project(workspace_root, project_dir)?;

    Ok(InitResult::Created { items: created })
}

/// Re-initialize an existing `.claude/` directory: recreate missing dirs/files.
pub fn reinit_project(project_dir: &Path, workspace_root: &Path) -> Result<Vec<String>, CoreError> {
    let claude = claude_dir(project_dir);
    let mut created = Vec::new();

    // Ensure settings.json exists with default hooks
    let settings_path = claude.join("settings.json");
    if !settings_path.exists() {
        let settings = default_settings();
        cc_schema::io::write_json(&settings_path, &settings)?;
        created.push(".claude/settings.json".to_string());
    } else if let Ok(mut settings) = cc_schema::io::read_json::<Settings>(&settings_path) {
        // Add default hooks if missing from existing settings
        if settings.hooks.is_none() {
            settings.hooks = Some(default_hooks());
            cc_schema::io::write_json(&settings_path, &settings)?;
            created.push(".claude/settings.json (added default hooks)".to_string());
        }
    }

    // Ensure resource directories exist
    let dirs = ["skills", "commands", "agents", "rules"];
    for dir in &dirs {
        let path = claude.join(dir);
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
            created.push(format!(".claude/{dir}/"));
        }
    }

    // Re-register in workspace
    WorkspaceConfig::set_current_project(workspace_root, project_dir)?;

    Ok(created)
}

/// Force-remove the existing `.claude/` directory and create a fresh one.
pub fn force_init_project(project_dir: &Path, workspace_root: &Path) -> Result<Vec<String>, CoreError> {
    let claude = claude_dir(project_dir);

    if claude.exists() {
        std::fs::remove_dir_all(&claude)?;
    }

    let created = create_fresh(&claude)?;
    WorkspaceConfig::set_current_project(workspace_root, project_dir)?;

    Ok(created)
}

/// Result of an init attempt.
pub enum InitResult {
    /// Fresh init succeeded — lists created items.
    Created { items: Vec<String> },
    /// `.claude/` already exists — describes what's there.
    AlreadyExists { existing: ExistingClaudeDir },
}

/// Description of an existing `.claude/` directory.
pub struct ExistingClaudeDir {
    pub has_settings: bool,
    pub skills_count: usize,
    pub commands_count: usize,
    pub agents_count: usize,
    pub rules_count: usize,
    pub hooks_count: usize,
}

impl ExistingClaudeDir {
    /// Total number of resources found.
    pub fn total_resources(&self) -> usize {
        self.skills_count + self.commands_count + self.agents_count + self.rules_count + self.hooks_count
    }

    /// Whether the existing directory looks like a valid init.
    pub fn is_valid_init(&self) -> bool {
        self.has_settings
    }
}

/// Scan an existing `.claude/` directory and report what's there.
fn scan_existing(claude_dir: &Path) -> ExistingClaudeDir {
    let has_settings = claude_dir.join("settings.json").exists();

    let skills_count = count_md_files(&claude_dir.join("skills"));
    let commands_count = count_md_files(&claude_dir.join("commands"));
    let agents_count = count_md_files(&claude_dir.join("agents"));
    let rules_count = count_md_files(&claude_dir.join("rules"));

    let hooks_count = cc_schema::io::read_json::<Settings>(&claude_dir.join("settings.json"))
        .ok()
        .and_then(|s| s.hooks)
        .map(|h| h.all_commands().len())
        .unwrap_or(0);

    ExistingClaudeDir {
        has_settings,
        skills_count,
        commands_count,
        agents_count,
        rules_count,
        hooks_count,
    }
}

/// Count .md files (including those in subdirectories like skill-name/SKILL.md).
fn count_md_files(dir: &Path) -> usize {
    if !dir.exists() {
        return 0;
    }
    walkdir(dir)
}

/// Recursively count .md files in a directory.
fn walkdir(dir: &Path) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else { return 0 };
    let mut count = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            count += walkdir(&path);
        } else if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        {
            count += 1;
        }
    }
    count
}

/// Create a fresh `.claude/` directory structure.
fn create_fresh(claude_dir: &Path) -> Result<Vec<String>, CoreError> {
    let mut created = Vec::new();

    std::fs::create_dir_all(claude_dir)?;
    created.push(".claude/".to_string());

    let settings = default_settings();
    let settings_path = claude_dir.join("settings.json");
    cc_schema::io::write_json(&settings_path, &settings)?;
    created.push(".claude/settings.json".to_string());

    let dirs = ["skills", "commands", "agents", "rules"];
    for dir in &dirs {
        let path = claude_dir.join(dir);
        std::fs::create_dir_all(&path)?;
        created.push(format!(".claude/{dir}/"));
    }

    Ok(created)
}

/// Build settings with default hooks.
fn default_settings() -> Settings {
    Settings {
        permissions: Default::default(),
        hooks: Some(default_hooks()),
    }
}

/// Build a default hook configuration.
fn default_hooks() -> HookConfig {
    let mut hooks = HashMap::new();

    hooks.insert(
        HookEvent::PreToolUse,
        vec![HookMatcher {
            matcher: Some("Bash".to_string()),
            hooks: vec![HookEntry {
                hook_type: "command".to_string(),
                command: "cargo fmt --check".to_string(),
            }],
        }],
    );

    hooks.insert(
        HookEvent::PostToolUse,
        vec![],
    );

    hooks.insert(
        HookEvent::Notification,
        vec![],
    );

    hooks.insert(
        HookEvent::Stop,
        vec![],
    );

    HookConfig { hooks }
}
