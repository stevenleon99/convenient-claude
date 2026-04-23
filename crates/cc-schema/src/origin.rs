use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

/// Where a resource originates from. Precedence: Project > User > External.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Origin {
    /// Community libraries in `extern/` (read-only, synced via git).
    External { library: String },
    /// User-wide defaults in `~/.claude/`.
    User,
    /// Project-specific configuration in `.claude/`.
    Project,
    /// Session-level overrides.
    Session,
}

impl Origin {
    /// Returns a display-friendly label for the origin.
    pub fn label(&self) -> String {
        match self {
            Origin::External { library } => format!("extern/{library}"),
            Origin::User => "user".to_string(),
            Origin::Project => "project".to_string(),
            Origin::Session => "session".to_string(),
        }
    }

    /// Precedence rank (higher = wins in conflict).
    pub fn precedence(&self) -> u8 {
        match self {
            Origin::External { .. } => 0,
            Origin::User => 1,
            Origin::Project => 2,
            Origin::Session => 3,
        }
    }

    /// Resolve the base directory for this origin.
    pub fn base_dir(&self, project_dir: &Path, user_claude_dir: &Path) -> PathBuf {
        match self {
            Origin::External { library } => project_dir.join("extern").join(library),
            Origin::User => user_claude_dir.to_path_buf(),
            Origin::Project => project_dir.join(".claude"),
            Origin::Session => project_dir.join(".claude"), // session files live in project
        }
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl Ord for Origin {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.precedence().cmp(&other.precedence())
    }
}

impl PartialOrd for Origin {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
