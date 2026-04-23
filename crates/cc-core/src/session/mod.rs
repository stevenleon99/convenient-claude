mod context;
mod modes;
mod tracker;

pub use context::SessionContext;
pub use modes::SessionMode;
pub use tracker::SessionTracker;

use cc_schema::SessionStats;
use std::path::Path;

/// Session state file name.
pub const SESSION_FILE: &str = "session.json";
/// Session stats file name.
pub const STATS_FILE: &str = "session-stats.json";
/// Stats history file name (JSONL).
pub const HISTORY_FILE: &str = "stats-history.jsonl";

/// Load active session info.
pub fn load_session(project_dir: &Path) -> Option<SessionContext> {
    let path = project_dir.join(".claude").join(SESSION_FILE);
    if path.exists() {
        let content = cc_schema::io::read_file(&path).ok()?;
        serde_json::from_str(&content).ok()
    } else {
        None
    }
}

/// Save session context to disk.
pub fn save_session(
    project_dir: &Path,
    ctx: &SessionContext,
) -> Result<(), cc_schema::SchemaError> {
    let path = project_dir.join(".claude").join(SESSION_FILE);
    cc_schema::io::write_json(&path, ctx)
}

/// Clear the active session.
pub fn clear_session(project_dir: &Path) -> std::io::Result<()> {
    let path = project_dir.join(".claude").join(SESSION_FILE);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    let stats_path = project_dir.join(".claude").join(STATS_FILE);
    if stats_path.exists() {
        std::fs::remove_file(stats_path)?;
    }
    Ok(())
}

/// Load session stats.
pub fn load_session_stats(project_dir: &Path) -> Option<SessionStats> {
    let path = project_dir.join(".claude").join(STATS_FILE);
    if path.exists() {
        cc_schema::io::read_json(&path).ok()
    } else {
        None
    }
}

/// Save session stats.
pub fn save_session_stats(
    project_dir: &Path,
    stats: &SessionStats,
) -> Result<(), cc_schema::SchemaError> {
    let path = project_dir.join(".claude").join(STATS_FILE);
    cc_schema::io::write_json(&path, stats)
}

/// Append session stats to history.
pub fn append_to_history(
    project_dir: &Path,
    stats: &SessionStats,
) -> Result<(), cc_schema::SchemaError> {
    let path = project_dir.join(".claude").join(HISTORY_FILE);
    cc_schema::io::append_jsonl(&path, stats)
}
