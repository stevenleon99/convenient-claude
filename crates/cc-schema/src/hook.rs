use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Hook configuration stored in `settings.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HookConfig {
    #[serde(default)]
    pub hooks: HashMap<HookEvent, Vec<HookMatcher>>,
}

/// Events that hooks can be registered for.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum HookEvent {
    PreToolUse,
    PostToolUse,
    Notification,
    Stop,
}

impl std::fmt::Display for HookEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookEvent::PreToolUse => write!(f, "PreToolUse"),
            HookEvent::PostToolUse => write!(f, "PostToolUse"),
            HookEvent::Notification => write!(f, "Notification"),
            HookEvent::Stop => write!(f, "Stop"),
        }
    }
}

/// A hook matcher: matches a tool name and runs associated hooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookMatcher {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matcher: Option<String>,
    pub hooks: Vec<HookEntry>,
}

/// A single hook entry (command to run).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookEntry {
    #[serde(rename = "type")]
    pub hook_type: String,
    pub command: String,
}

impl HookConfig {
    /// Parse hooks from a JSON value (extracted from settings.json).
    pub fn from_json_value(value: &serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value.clone())
    }

    /// Get all unique commands across all events and matchers.
    pub fn all_commands(&self) -> Vec<&str> {
        let mut commands: Vec<&str> = Vec::new();
        for matchers in self.hooks.values() {
            for matcher in matchers {
                for entry in &matcher.hooks {
                    commands.push(&entry.command);
                }
            }
        }
        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hook_config() {
        let json = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {
                        "matcher": "Bash",
                        "hooks": [
                            { "type": "command", "command": "lint-check.sh" }
                        ]
                    }
                ],
                "PostToolUse": [],
                "Notification": [],
                "Stop": []
            }
        });

        let config: HookConfig = serde_json::from_value(json).unwrap();
        assert!(config.hooks.contains_key(&HookEvent::PreToolUse));
        let pre_hooks = &config.hooks[&HookEvent::PreToolUse];
        assert_eq!(pre_hooks.len(), 1);
        assert_eq!(pre_hooks[0].matcher.as_deref(), Some("Bash"));
        assert_eq!(pre_hooks[0].hooks[0].command, "lint-check.sh");
    }

    #[test]
    fn test_all_commands() {
        let config = HookConfig {
            hooks: {
                let mut map = HashMap::new();
                map.insert(
                    HookEvent::PreToolUse,
                    vec![HookMatcher {
                        matcher: Some("Bash".into()),
                        hooks: vec![
                            HookEntry {
                                hook_type: "command".into(),
                                command: "a.sh".into(),
                            },
                            HookEntry {
                                hook_type: "command".into(),
                                command: "b.sh".into(),
                            },
                        ],
                    }],
                );
                map
            },
        };
        let cmds = config.all_commands();
        assert_eq!(cmds.len(), 2);
    }
}
