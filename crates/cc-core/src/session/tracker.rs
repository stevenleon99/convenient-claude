use cc_schema::{ResourceType, SessionMode, SessionStats};

/// Tracks resource and token usage for an active session.
#[derive(Debug, Clone)]
pub struct SessionTracker {
    stats: SessionStats,
}

impl SessionTracker {
    /// Create a new tracker for a session.
    pub fn new(session_id: String, mode: SessionMode) -> Self {
        Self {
            stats: SessionStats::new(session_id, mode),
        }
    }

    /// Record a tool invocation.
    pub fn record_tool(&mut self, tool_name: &str) {
        self.stats.record_tool_use(tool_name);
    }

    /// Add tokens to the session total.
    pub fn add_tokens(&mut self, input: u64, output: u64) {
        self.stats.add_tokens(input, output);
    }

    /// Record usage for a specific resource.
    pub fn record_resource_usage(
        &mut self,
        resource_type: ResourceType,
        name: &str,
        origin: cc_schema::Origin,
        tokens: u64,
    ) {
        // Find existing entry or create new one
        if let Some(existing) = self
            .stats
            .resource_usage
            .iter_mut()
            .find(|r| r.resource_type == resource_type && r.name == name)
        {
            existing.times_invoked += 1;
            existing.tokens_consumed += tokens;
        } else {
            self.stats.resource_usage.push(cc_schema::ResourceUsage {
                resource_type,
                name: name.to_string(),
                origin,
                times_invoked: 1,
                tokens_consumed: tokens,
            });
        }
    }

    /// Get the current stats.
    pub fn stats(&self) -> &SessionStats {
        &self.stats
    }

    /// Get mutable stats.
    pub fn stats_mut(&mut self) -> &mut SessionStats {
        &mut self.stats
    }

    /// Finalize the session (mark as stopped).
    pub fn finalize(&mut self) {
        self.stats.stop();
    }
}
