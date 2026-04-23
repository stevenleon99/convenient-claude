use super::load_stats_history;
use cc_schema::{ResourceType, SessionStats};
use std::collections::HashMap;
use std::path::Path;

/// Per-resource usage summary aggregated across sessions.
#[derive(Debug, Default)]
pub struct ResourceSummary {
    pub total_sessions: u32,
    pub total_tokens: u64,
    pub total_invocations: u32,
    pub total_cost: f64,
}

/// Load and aggregate resource usage from history.
pub fn aggregate_resource_usage(
    project_dir: &Path,
    last_n: Option<usize>,
) -> HashMap<(ResourceType, String), ResourceSummary> {
    let history = load_stats_history(project_dir);

    let sessions: Vec<&SessionStats> = match last_n {
        Some(n) => history.iter().rev().take(n).collect(),
        None => history.iter().collect(),
    };

    let mut summary: HashMap<(ResourceType, String), ResourceSummary> = HashMap::new();

    for session in &sessions {
        for usage in &session.resource_usage {
            let key = (usage.resource_type, usage.name.clone());
            let entry = summary.entry(key).or_default();
            entry.total_sessions += 1;
            entry.total_tokens += usage.tokens_consumed;
            entry.total_invocations += usage.times_invoked;
        }
    }

    summary
}
