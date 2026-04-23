use cc_schema::Origin;
use std::path::{Path, PathBuf};

/// Resolve the user's `~/.claude/` directory.
pub fn user_claude_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
}

/// Resolve the project `.claude/` directory by searching upward from `start_dir`.
pub fn find_project_dir(start_dir: &Path) -> Option<PathBuf> {
    let mut dir = start_dir.to_path_buf();
    loop {
        let claude_dir = dir.join(".claude");
        if claude_dir.is_dir() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Check whether a project is initialized (has `.claude/` directory).
pub fn is_initialized(project_dir: &Path) -> bool {
    project_dir.join(".claude").is_dir()
}

/// Get the `.claude/` directory for a project.
pub fn claude_dir(project_dir: &Path) -> PathBuf {
    project_dir.join(".claude")
}

/// Get the resource directory for a given resource type within a project.
pub fn resource_dir(project_dir: &Path, resource_type: cc_schema::ResourceType) -> PathBuf {
    let dir = resource_type.dir_name();
    if dir.is_empty() {
        claude_dir(project_dir)
    } else {
        claude_dir(project_dir).join(dir)
    }
}

/// Resolve the absolute directory for an origin + resource type.
pub fn origin_resource_dir(
    origin: &Origin,
    resource_type: cc_schema::ResourceType,
    project_dir: &Path,
) -> PathBuf {
    let base = origin.base_dir(project_dir, &user_claude_dir());
    let dir = resource_type.dir_name();
    if dir.is_empty() {
        base
    } else {
        base.join(dir)
    }
}

/// Simple `dirs::home_dir()` since we don't want to pull in the `dirs` crate.
mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .ok()
            .map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_claude_dir() {
        let dir = user_claude_dir();
        assert!(dir.to_string_lossy().contains(".claude"));
    }

    #[test]
    fn test_is_initialized_false() {
        assert!(!is_initialized(Path::new("/nonexistent")));
    }

    #[test]
    fn test_resource_dir_skill() {
        let dir = resource_dir(Path::new("/project"), cc_schema::ResourceType::Skill);
        assert_eq!(dir, PathBuf::from("/project/.claude/skills"));
    }

    #[test]
    fn test_resource_dir_hook() {
        let dir = resource_dir(Path::new("/project"), cc_schema::ResourceType::Hook);
        assert_eq!(dir, PathBuf::from("/project/.claude"));
    }
}
