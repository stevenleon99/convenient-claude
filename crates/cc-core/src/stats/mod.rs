mod token;
mod usage;

pub use token::estimate_cost;
pub use usage::aggregate_resource_usage;

use cc_schema::SessionStats;
use std::path::Path;

/// Load historical session stats from the JSONL file.
pub fn load_stats_history(project_dir: &Path) -> Vec<SessionStats> {
    let path = project_dir.join(".claude").join("stats-history.jsonl");
    if !path.exists() {
        return Vec::new();
    }

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<SessionStats>(line).ok())
        .collect()
}
