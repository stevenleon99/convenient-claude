use cc_schema::ResourceType;
use clap::{Parser, Subcommand};

/// convenient-claude — One stop place to setup Claude for your project.
#[derive(Parser)]
#[command(name = "cc", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Show verbose output.
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress non-error output.
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Override project directory detection.
    #[arg(short, long, global = true)]
    pub project_dir: Option<String>,

    /// Disable colored output.
    #[arg(long, global = true)]
    pub no_color: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize .claude/ in a project.
    Init,

    /// List resources.
    List {
        #[command(subcommand)]
        resource: ListTarget,
    },

    /// Install a resource.
    Add {
        #[command(subcommand)]
        resource: AddTarget,
    },

    /// Uninstall a resource.
    Remove {
        #[command(subcommand)]
        resource: RemoveTarget,
    },

    /// Display resource details + origin.
    Show {
        /// Resource type.
        resource_type: String,
        /// Resource name.
        name: String,
    },

    /// Validate all project resources.
    Validate {
        /// Auto-fix what's fixable.
        #[arg(long)]
        fix: bool,
    },

    /// Sync extern/ submodules.
    Sync {
        /// Preview changes without applying.
        #[arg(long)]
        dry_run: bool,
    },

    /// Session management.
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },

    /// View/edit merged configuration.
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Resource and token usage analytics.
    Stats {
        #[command(subcommand)]
        action: StatsAction,
    },

    /// Diagnose setup issues.
    Doctor,

    /// Generate shell completions.
    Completions {
        /// Target shell.
        shell: String,
    },

    /// Launch interactive TUI dashboard.
    Tui,
}

#[derive(Subcommand)]
pub enum ListTarget {
    /// List skills.
    Skills {
        /// Filter by name or description.
        filter: Option<String>,
        /// Output format.
        #[arg(long, default_value = "table")]
        format: OutputFormat,
    },
    /// List commands.
    Commands {
        filter: Option<String>,
        #[arg(long, default_value = "table")]
        format: OutputFormat,
    },
    /// List agents.
    Agents {
        filter: Option<String>,
        #[arg(long, default_value = "table")]
        format: OutputFormat,
    },
    /// List hooks.
    Hooks {
        #[arg(long, default_value = "table")]
        format: OutputFormat,
    },
    /// List rules.
    Rules {
        filter: Option<String>,
        #[arg(long, default_value = "table")]
        format: OutputFormat,
    },
    /// List local plugins.
    Plugins {
        #[arg(long, default_value = "table")]
        format: OutputFormat,
    },
    /// List everything, grouped by type.
    All {
        #[arg(long, default_value = "table")]
        format: OutputFormat,
    },
}

#[derive(Subcommand)]
pub enum AddTarget {
    /// Install a skill.
    Skill {
        /// Skill name.
        name: String,
        /// Source: extern/<lib>, url, file path.
        #[arg(long)]
        from: Option<String>,
        /// Target: project (default), user.
        #[arg(long, default_value = "project")]
        to: String,
        /// Overwrite if exists.
        #[arg(long)]
        force: bool,
    },
    /// Install a command.
    Command {
        name: String,
        #[arg(long)]
        from: Option<String>,
        #[arg(long, default_value = "project")]
        to: String,
        #[arg(long)]
        force: bool,
    },
    /// Install an agent.
    Agent {
        name: String,
        #[arg(long)]
        from: Option<String>,
        #[arg(long, default_value = "project")]
        to: String,
        #[arg(long)]
        force: bool,
    },
    /// Register a hook.
    Hook {
        /// Event: PreToolUse, PostToolUse, Notification, Stop.
        event: String,
        /// Command to run.
        cmd: String,
        /// Tool matcher pattern.
        #[arg(long)]
        matcher: Option<String>,
    },
    /// Install a rule.
    Rule {
        name: String,
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
pub enum RemoveTarget {
    /// Remove a skill.
    Skill { name: String },
    /// Remove a command.
    Command { name: String },
    /// Remove an agent.
    Agent { name: String },
    /// Remove a hook.
    Hook {
        /// Event: PreToolUse, PostToolUse, Notification, Stop.
        event: String,
        /// Command to remove.
        cmd: String,
    },
    /// Remove a rule.
    Rule { name: String },
}

#[derive(Subcommand)]
pub enum SessionAction {
    /// Start a managed session.
    Start {
        /// conversation | loop | interactive.
        #[arg(long, default_value = "interactive")]
        mode: String,
        /// Activate specific skills (comma-separated).
        #[arg(long, value_delimiter = ',')]
        skills: Vec<String>,
        /// Activate specific agents (comma-separated).
        #[arg(long, value_delimiter = ',')]
        agents: Vec<String>,
    },
    /// Stop active session.
    Stop,
    /// Show active session info.
    Status,
    /// Show resource and token usage stats.
    Stats,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Print effective (merged) settings.
    Show,
    /// Get a specific config value.
    Get { key: String },
    /// Set a config value.
    Set {
        key: String,
        value: String,
        /// project | user.
        #[arg(long, default_value = "project")]
        scope: String,
    },
    /// Show overrides vs. user defaults.
    Diff,
}

#[derive(Subcommand)]
pub enum StatsAction {
    /// Current session stats (live).
    Session,
    /// Historical session stats.
    History {
        /// Last N sessions.
        #[arg(long, default_value = "10")]
        last: usize,
    },
    /// Per-resource usage breakdown.
    Resources,
}

#[derive(Clone, Debug)]
pub enum OutputFormat {
    Table,
    Json,
    Plain,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "table" => Ok(OutputFormat::Table),
            "json" => Ok(OutputFormat::Json),
            "plain" => Ok(OutputFormat::Plain),
            _ => Err(format!(
                "unknown output format: {s} (expected table, json, or plain)"
            )),
        }
    }
}

/// Parse a resource type from a string.
pub fn parse_resource_type(s: &str) -> Option<ResourceType> {
    match s.to_lowercase().as_str() {
        "skill" | "skills" => Some(ResourceType::Skill),
        "command" | "commands" => Some(ResourceType::Command),
        "agent" | "agents" => Some(ResourceType::Agent),
        "hook" | "hooks" => Some(ResourceType::Hook),
        "rule" | "rules" => Some(ResourceType::Rule),
        _ => None,
    }
}
