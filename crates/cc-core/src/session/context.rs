use cc_schema::{ResourceType, SessionMode};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Active session context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    /// Unique session ID.
    pub session_id: String,
    /// When the session started.
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Session execution mode.
    pub mode: SessionMode,
    /// Active skill names.
    pub active_skills: HashSet<String>,
    /// Active agent names.
    pub active_agents: HashSet<String>,
    /// Active command names.
    pub active_commands: HashSet<String>,
    /// Active hooks.
    pub active_hooks: Vec<String>,
}

impl SessionContext {
    /// Create a new session context.
    pub fn new(mode: SessionMode) -> Self {
        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            started_at: chrono::Utc::now(),
            mode,
            active_skills: HashSet::new(),
            active_agents: HashSet::new(),
            active_commands: HashSet::new(),
            active_hooks: Vec::new(),
        }
    }

    /// Activate skills by name.
    pub fn activate_skills(&mut self, skills: &[String]) {
        for skill in skills {
            self.active_skills.insert(skill.clone());
        }
    }

    /// Activate agents by name.
    pub fn activate_agents(&mut self, agents: &[String]) {
        for agent in agents {
            self.active_agents.insert(agent.clone());
        }
    }

    /// Activate commands by name.
    pub fn activate_commands(&mut self, commands: &[String]) {
        for cmd in commands {
            self.active_commands.insert(cmd.clone());
        }
    }

    /// Check if a resource is active in this session.
    pub fn is_resource_active(&self, resource_type: ResourceType, name: &str) -> bool {
        match resource_type {
            ResourceType::Skill => self.active_skills.contains(name),
            ResourceType::Agent => self.active_agents.contains(name),
            ResourceType::Command => self.active_commands.contains(name),
            _ => false,
        }
    }
}
