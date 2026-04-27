use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Parsed cc-workspace.toml configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkspaceConfig {
    #[serde(default, rename = "local-claude-plugin")]
    pub local_claude_plugin: LocalPluginConfig,
    #[serde(default, rename = "local-other-plugin")]
    pub local_other_plugin: OtherPluginConfig,
    #[serde(default)]
    pub external: ExternalConfig,
    #[serde(default)]
    pub local: LocalConfig,
    #[serde(default, rename = "current project")]
    pub project: ProjectConfig,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LocalPluginConfig {
    #[serde(default)]
    pub claude_plugins: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct OtherPluginConfig {
    #[serde(default)]
    pub other_plugins: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ExternalConfig {
    #[serde(default)]
    pub projects: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LocalConfig {
    #[serde(default)]
    pub path: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProjectConfig {
    #[serde(default)]
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
        // Use forward slashes so TOML doesn't interpret backslashes as escape sequences
        let path_value = path_value.replace('\\', "/");

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

    /// Resolve subdirectories under `other_plugins` as resource registries.
    /// Each subdirectory (e.g., fullstack-dev-skills, obsidian-skills) becomes
    /// a registry whose path contains skills/, commands/, agents/, rules/ etc.
    pub fn other_plugin_registries(&self) -> Vec<Registry> {
        let Some(other) = &self.local_other_plugin.other_plugins else {
            return Vec::new();
        };

        let expanded = expand_tilde(other);
        if !expanded.is_dir() {
            return Vec::new();
        }

        let mut registries = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&expanded) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        if !name.starts_with('.') {
                            registries.push(Registry {
                                label: name.to_string(),
                                path,
                            });
                        }
                    }
                }
            }
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

    /// Resolve only the claude_plugins directories from [local-claude-plugin].
    /// Returns (path, source_label) pairs.
    pub fn claude_plugin_dirs(&self) -> Vec<(PathBuf, String)> {
        let user_claude = if let Some(home) = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .ok()
        {
            PathBuf::from(home).join(".claude")
        } else {
            PathBuf::new()
        };

        self.local_claude_plugin
            .claude_plugins
            .iter()
            .map(|plugin_dir| {
                let expanded = expand_tilde(plugin_dir);
                let resolved = if expanded.is_absolute() {
                    expanded
                } else {
                    PathBuf::from(plugin_dir)
                };
                let source = path_relative_to(&resolved, &user_claude);
                (resolved, source)
            })
            .collect()
    }

    /// Resolve all plugin directories from local-claude-plugin and local-other-plugin sections.
    /// Returns (path, source_label) pairs where source_label is a path relative to
    /// ~/.claude/plugins/marketplaces/ (e.g. "claude-plugins-official/plugins").
    pub fn plugin_dirs(&self, workspace_root: &Path) -> Vec<(PathBuf, String)> {
        let mut dirs = Vec::new();

        let user_claude = if let Some(home) = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .ok()
        {
            PathBuf::from(home).join(".claude")
        } else {
            PathBuf::new()
        };

        // From [local-claude-plugin].claude_plugins — each path is a directory containing plugins
        for plugin_dir in &self.local_claude_plugin.claude_plugins {
            let expanded = expand_tilde(plugin_dir);
            let resolved = if expanded.is_absolute() {
                expanded
            } else {
                workspace_root.join(&expanded)
            };
            let source = path_relative_to(&resolved, &user_claude);
            dirs.push((resolved, source));
        }

        // From [local-other-plugin].other_plugins — scan subdirectories of this path
        if let Some(other) = &self.local_other_plugin.other_plugins {
            let expanded = expand_tilde(other);
            let resolved = if expanded.is_absolute() {
                expanded
            } else {
                workspace_root.join(&expanded)
            };
            // Each subdirectory is a plugin marketplace
            if resolved.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&resolved) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            if let Some(name) = entry.file_name().to_str() {
                                if !name.starts_with('.') {
                                    let source = path_relative_to(&path, &user_claude);
                                    dirs.push((path, source));
                                }
                            }
                        }
                    }
                }
            }
        }

        dirs
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

/// Return `path` relative to `base`, using forward slashes. Falls back to the
/// file name if `path` is not under `base`.
fn path_relative_to(path: &Path, base: &Path) -> String {
    if let Ok(rel) = path.strip_prefix(base) {
        rel.to_string_lossy().replace('\\', "/")
    } else {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("plugin")
            .to_string()
    }
}

const DEFAULT_WORKSPACE_TOML: &str = r#"# cc-workspace.toml — Workspace registry for convenient-claude

[external]
projects = []

[local]
path = "~/.claude"

["current project"]
path = ""
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Test that the actual cc-workspace.toml in this repo parses successfully.
    #[test]
    fn test_parse_real_workspace_config() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let config = WorkspaceConfig::load(workspace_root)
            .expect("cc-workspace.toml should parse without error");

        // Should have plugin directories from [local-claude-plugin]
        assert!(
            !config.local_claude_plugin.claude_plugins.is_empty(),
            "claude_plugins should not be empty"
        );

        // Should have other_plugins from [local-other-plugin]
        assert!(
            config.local_other_plugin.other_plugins.is_some(),
            "other_plugins should be set"
        );
    }

    /// Test that claude_plugin_dirs returns exactly the two configured directories.
    #[test]
    fn test_claude_plugin_dirs_resolve_to_existing_directories() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let config = WorkspaceConfig::load(workspace_root)
            .expect("cc-workspace.toml should parse");

        let dirs = config.claude_plugin_dirs();
        assert_eq!(
            dirs.len(),
            2,
            "should have exactly 2 claude_plugin directories"
        );

        for (path, label) in &dirs {
            assert!(
                path.is_dir(),
                "plugin dir {} ({}) should exist on disk",
                label,
                path.display()
            );
        }
    }

    /// Test that claude_plugin_dirs discovers actual plugin subdirectories.
    #[test]
    fn test_claude_plugin_dirs_contain_plugins() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let config = WorkspaceConfig::load(workspace_root)
            .expect("cc-workspace.toml should parse");

        let dirs = config.claude_plugin_dirs();
        let mut total_plugins = 0;

        for (dir, label) in &dirs {
            if dir.is_dir() {
                let entries: Vec<_> = fs::read_dir(dir)
                    .unwrap()
                    .flatten()
                    .filter(|e| e.path().is_dir())
                    .collect();
                total_plugins += entries.len();
                println!(
                    "  source {} -> {} plugins",
                    label,
                    entries.len()
                );
            }
        }

        assert!(
            total_plugins > 0,
            "claude_plugin dirs should contain plugins, found {total_plugins}"
        );
    }

    /// Test that missing sections in TOML don't break parsing.
    #[test]
    fn test_minimal_config_parses() {
        let toml_str = r#"
[local-claude-plugin]
claude_plugins = []

[local-other-plugin]
"#;
        let config: WorkspaceConfig = toml::from_str(toml_str).expect("minimal config should parse");
        assert!(config.local_claude_plugin.claude_plugins.is_empty());
        assert!(config.external.projects.is_empty());
        assert!(config.local.path.is_empty());
        assert!(config.project.path.is_empty());
    }
}
