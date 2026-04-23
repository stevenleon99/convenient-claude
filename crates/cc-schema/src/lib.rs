pub mod agent;
pub mod command;
pub mod error;
pub mod frontmatter;
pub mod hook;
pub mod io;
pub mod origin;
pub mod resource_type;
pub mod rule;
pub mod settings;
pub mod skill;
pub mod stats;

// Re-export primary types for convenience.
pub use agent::Agent;
pub use command::Command;
pub use error::SchemaError;
pub use hook::{HookConfig, HookEntry, HookEvent, HookMatcher};
pub use origin::Origin;
pub use resource_type::ResourceType;
pub use rule::Rule;
pub use settings::{Permissions, Settings};
pub use skill::{Skill, SkillMetadata};
pub use stats::{ResourceUsage, SessionMode, SessionStats, TokenBreakdown, TokenUsage};
