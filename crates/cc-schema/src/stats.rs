use crate::origin::Origin;
use crate::resource_type::ResourceType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tracked statistics for a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub session_id: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub stopped_at: Option<chrono::DateTime<chrono::Utc>>,
    pub mode: SessionMode,
    pub token_usage: TokenUsage,
    pub resource_usage: Vec<ResourceUsage>,
    pub tool_invocations: HashMap<String, u32>,
}

/// Token usage tracking.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    /// Estimated cost in USD.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_cost: Option<f64>,
    /// Token breakdown by resource type.
    #[serde(default)]
    pub by_resource: HashMap<ResourceType, TokenBreakdown>,
}

/// Token breakdown for a specific resource type.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenBreakdown {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// Usage stats for a specific resource within a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub resource_type: ResourceType,
    pub name: String,
    pub origin: Origin,
    pub times_invoked: u32,
    pub tokens_consumed: u64,
}

/// Session execution mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionMode {
    Conversation,
    Loop,
    Interactive,
}

impl SessionStats {
    /// Create a new empty session stats with the given ID and mode.
    pub fn new(session_id: String, mode: SessionMode) -> Self {
        SessionStats {
            session_id,
            started_at: chrono::Utc::now(),
            stopped_at: None,
            mode,
            token_usage: TokenUsage::default(),
            resource_usage: Vec::new(),
            tool_invocations: HashMap::new(),
        }
    }

    /// Record a tool invocation.
    pub fn record_tool_use(&mut self, tool_name: &str) {
        *self
            .tool_invocations
            .entry(tool_name.to_string())
            .or_insert(0) += 1;
    }

    /// Add tokens to the session total.
    pub fn add_tokens(&mut self, input: u64, output: u64) {
        self.token_usage.input_tokens += input;
        self.token_usage.output_tokens += output;
        self.token_usage.total_tokens += input + output;
    }

    /// Mark the session as stopped.
    pub fn stop(&mut self) {
        self.stopped_at = Some(chrono::Utc::now());
    }

    /// Duration of the session (or until now if still running).
    pub fn duration(&self) -> chrono::Duration {
        let end = self.stopped_at.unwrap_or_else(chrono::Utc::now);
        end - self.started_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_stats_new() {
        let stats = SessionStats::new("test-id".into(), SessionMode::Interactive);
        assert_eq!(stats.session_id, "test-id");
        assert!(stats.stopped_at.is_none());
        assert_eq!(stats.token_usage.total_tokens, 0);
    }

    #[test]
    fn test_record_tool_use() {
        let mut stats = SessionStats::new("test".into(), SessionMode::Loop);
        stats.record_tool_use("Bash");
        stats.record_tool_use("Bash");
        stats.record_tool_use("Read");
        assert_eq!(stats.tool_invocations["Bash"], 2);
        assert_eq!(stats.tool_invocations["Read"], 1);
    }

    #[test]
    fn test_add_tokens() {
        let mut stats = SessionStats::new("test".into(), SessionMode::Conversation);
        stats.add_tokens(100, 50);
        stats.add_tokens(200, 75);
        assert_eq!(stats.token_usage.input_tokens, 300);
        assert_eq!(stats.token_usage.output_tokens, 125);
        assert_eq!(stats.token_usage.total_tokens, 425);
    }
}
