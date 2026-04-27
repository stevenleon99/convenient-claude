use serde::{Deserialize, Serialize};
use std::fmt;

/// The types of Claude Code resources managed by this tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    Skill,
    Command,
    Agent,
    Hook,
    Rule,
    Plugin,
}

impl ResourceType {
    /// Directory name within `.claude/` for this resource type.
    pub fn dir_name(&self) -> &'static str {
        match self {
            ResourceType::Skill => "skills",
            ResourceType::Command => "commands",
            ResourceType::Agent => "agents",
            ResourceType::Hook => "", // hooks live in settings.json
            ResourceType::Rule => "rules",
            ResourceType::Plugin => "plugins",
        }
    }

    /// File extension for this resource type.
    pub fn extension(&self) -> &'static str {
        match self {
            ResourceType::Hook => "json",
            _ => "md",
        }
    }

    /// Iterate over all resource types.
    pub fn all() -> &'static [ResourceType] {
        &[
            ResourceType::Skill,
            ResourceType::Command,
            ResourceType::Agent,
            ResourceType::Hook,
            ResourceType::Rule,
            ResourceType::Plugin,
        ]
    }
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceType::Skill => write!(f, "skill"),
            ResourceType::Command => write!(f, "command"),
            ResourceType::Agent => write!(f, "agent"),
            ResourceType::Hook => write!(f, "hook"),
            ResourceType::Rule => write!(f, "rule"),
            ResourceType::Plugin => write!(f, "plugin"),
        }
    }
}
