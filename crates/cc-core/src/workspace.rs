use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Parsed cc-workspace.toml configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkspaceConfig {
    pub external: ExternalConfig,
    pub local: LocalConfig,
    #[serde(rename = "current project")]
    pub project: ProjectConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExternalConfig {
    pub projects: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LocalConfig {
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectConfig {
    pub path: String,
}

/// A resolved registry entry with an absolute path and a label.
#[derive(Debug, Clone)]
pub struct Registry {
    /// Human-readable label (e.g. "extern/claude-skills", "local", "project").
    pub label: String,
    /// Absolute path to the registry root directory.
    pub path: PathBuf,
}

impl WorkspaceConfig {
    /// Load workspace config from the `resource/cc-workspace.toml` file.
    pub fn load(workspace_root: &Path) -> Option<Self> {
        let config_path = workspace_root.join("resource").join("cc-workspace.toml");
        if !config_path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(&config_path).ok()?;
        toml::from_str(&content).ok()
    }

    /// Save workspace config back to `resource/cc-workspace.toml`.
    pub fn save(&self, workspace_root: &Path) -> Result<(), std::io::Error> {
        let config_path = workspace_root.join("resource").join("cc-workspace.toml");
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e)
        })?;
        std::fs::write(&config_path, content)
    }

    /// Set the current project path by patching the TOML file in-place (preserves comments).
    pub fn set_current_project(
        workspace_root: &Path,
        project_dir: &Path,
    ) -> Result<(), std::io::Error> {
        let config_path = workspace_root.join("resource").join("cc-workspace.toml");

        // Ensure the file exists
        if !config_path.exists() {
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&config_path, DEFAULT_WORKSPACE_TOML)?;
        }

        let content = std::fs::read_to_string(&config_path)?;
        let path_value = project_dir.to_string_lossy();
        // Strip Windows \\?\ prefix for clean paths
        let path_value = path_value.strip_prefix(r"\\?\").unwrap_or(&path_value);

        // Replace the path = "..." line under [current project] or ["current project"]
        let mut new_lines = Vec::new();
        let mut in_project_section = false;

        for line in content.lines() {
            let trimmed = line.trim();
            // Detect section headers
            if trimmed.starts_with('[') {
                let section = trimmed
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .trim()
                    .trim_matches('"');
                in_project_section = section == "current project";
            }
            // Replace path line inside the project section
            if in_project_section && trimmed.starts_with("path") {
                new_lines.push(format!("path = \"{path_value}\""));
                in_project_section = false; // only replace the first match
            } else {
                new_lines.push(line.to_string());
            }
        }

        std::fs::write(&config_path, new_lines.join("\n") + "\n")
    }

    /// Resolve all registries to absolute paths with labels.
    pub fn registries(&self, workspace_root: &Path) -> Vec<Registry> {
        let mut registries = Vec::new();

        // External registries
        for proj in &self.external.projects {
            let path = if Path::new(proj).is_absolute() {
                PathBuf::from(proj)
            } else {
                workspace_root.join(proj)
            };
            registries.push(Registry {
                label: proj.clone(),
                path,
            });
        }

        // Local (user) registry
        let local_path = expand_tilde(&self.local.path);
        registries.push(Registry {
            label: "local".to_string(),
            path: local_path,
        });

        // Current project registry
        if !self.project.path.is_empty() {
            let project_path = if Path::new(&self.project.path).is_absolute() {
                PathBuf::from(&self.project.path)
            } else {
                workspace_root.join(&self.project.path)
            };
            registries.push(Registry {
                label: "project".to_string(),
                path: project_path,
            });
        }

        registries
    }

    /// Get the current project path from the config.
    pub fn current_project_path(&self, workspace_root: &Path) -> Option<PathBuf> {
        if self.project.path.is_empty() {
            return None;
        }
        let path = if Path::new(&self.project.path).is_absolute() {
            PathBuf::from(&self.project.path)
        } else {
            workspace_root.join(&self.project.path)
        };
        Some(path)
    }
}

/// Expand `~` to the user's home directory.
fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .ok()
        {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(path)
}

const DEFAULT_WORKSPACE_TOML: &str = r#"# cc-workspace.toml — Workspace registry for convenient-claude

[external]
projects = []

[local]
path = "~/.claude"

["current project"]
path = ""
"#;
