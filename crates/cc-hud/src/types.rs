use serde::Deserialize;

/// JSON data piped from Claude Code via stdin every ~300ms.
#[derive(Deserialize, Default, Debug, Clone)]
pub struct StdinData {
    pub transcript_path: Option<String>,
    pub cwd: Option<String>,
    pub model: Option<ModelInfo>,
    pub context_window: Option<ContextWindow>,
    pub cost: Option<CostInfo>,
    pub rate_limits: Option<RateLimits>,
    pub effort: Option<EffortValue>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ModelInfo {
    pub id: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct ContextWindow {
    pub context_window_size: Option<u64>,
    pub total_input_tokens: Option<u64>,
    pub current_usage: Option<TokenUsage>,
    pub used_percentage: Option<f64>,
    pub remaining_percentage: Option<f64>,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct TokenUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cache_creation_input_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct CostInfo {
    pub total_cost_usd: Option<f64>,
    pub total_duration_ms: Option<u64>,
    pub total_api_duration_ms: Option<u64>,
    pub total_lines_added: Option<u64>,
    pub total_lines_removed: Option<u64>,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct RateLimits {
    pub five_hour: Option<RateWindow>,
    pub seven_day: Option<RateWindow>,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct RateWindow {
    pub used_percentage: Option<f64>,
    pub resets_at: Option<f64>,
}

/// Effort can be a string, an object { level: "max" }, or null.
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum EffortValue {
    Level { level: Option<String> },
    Bare(String),
}

// ---------------------------------------------------------------------------
// Transcript types (parsed from JSONL transcript file)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct TranscriptData {
    pub tools: Vec<ToolEntry>,
    pub agents: Vec<AgentEntry>,
    pub todos: Vec<TodoItem>,
    pub session_start: Option<chrono::DateTime<chrono::Utc>>,
    pub session_name: Option<String>,
    pub last_assistant_response_at: Option<chrono::DateTime<chrono::Utc>>,
    pub session_tokens: Option<SessionTokenUsage>,
    pub last_compact_boundary_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_compact_post_tokens: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct ToolEntry {
    pub id: String,
    pub name: String,
    pub target: Option<String>,
    pub status: ToolStatus,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolStatus {
    Running,
    Completed,
    Error,
}

#[derive(Debug, Clone)]
pub struct AgentEntry {
    pub id: String,
    pub agent_type: String,
    pub model: Option<String>,
    pub description: Option<String>,
    pub status: ToolStatus,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct TodoItem {
    pub content: String,
    pub status: TodoStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, Default)]
pub struct SessionTokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
}

impl SessionTokenUsage {
    pub fn total(&self) -> u64 {
        self.input_tokens + self.output_tokens + self.cache_creation_tokens + self.cache_read_tokens
    }
}

// ---------------------------------------------------------------------------
// Git status
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct GitStatus {
    pub branch: String,
    pub is_dirty: bool,
    pub ahead: u32,
    pub behind: u32,
}

// ---------------------------------------------------------------------------
// Usage data (rate limits)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct UsageData {
    pub five_hour: Option<f64>,
    pub seven_day: Option<f64>,
    pub five_hour_reset_at: Option<chrono::DateTime<chrono::Utc>>,
    pub seven_day_reset_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl UsageData {
    pub fn is_limit_reached(&self) -> bool {
        self.five_hour == Some(100.0) || self.seven_day == Some(100.0)
    }
}

// ---------------------------------------------------------------------------
// Render context
// ---------------------------------------------------------------------------

/// Display configuration for the HUD output.
#[derive(Debug, Clone)]
pub struct HudConfig {
    pub layout: Layout,
    pub show_model: bool,
    pub show_context_bar: bool,
    pub show_project: bool,
    pub show_git: bool,
    pub show_tools: bool,
    pub show_agents: bool,
    pub show_todos: bool,
    pub show_usage: bool,
    pub show_duration: bool,
    pub show_config_counts: bool,
    pub show_cost: bool,
    pub show_session_tokens: bool,
    pub context_bar_width: usize,
    pub path_levels: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Compact,
    Expanded,
}

impl Default for HudConfig {
    fn default() -> Self {
        Self {
            layout: Layout::Expanded,
            show_model: true,
            show_context_bar: true,
            show_project: true,
            show_git: true,
            show_tools: true,
            show_agents: true,
            show_todos: true,
            show_usage: true,
            show_duration: true,
            show_config_counts: true,
            show_cost: true,
            show_session_tokens: false,
            context_bar_width: 10,
            path_levels: 1,
        }
    }
}

/// Everything needed to render the HUD.
#[derive(Debug, Clone)]
pub struct RenderContext {
    pub stdin: StdinData,
    pub transcript: TranscriptData,
    pub claude_md_count: usize,
    pub rules_count: usize,
    pub mcp_count: usize,
    pub hooks_count: usize,
    pub git_status: Option<GitStatus>,
    pub usage_data: UsageData,
    pub config: HudConfig,
}
