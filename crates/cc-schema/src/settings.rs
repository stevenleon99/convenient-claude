use crate::hook::HookConfig;
use serde::{Deserialize, Serialize};

/// Claude Code settings stored in `settings.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub permissions: Permissions,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hooks: Option<HookConfig>,
}

/// Permission rules for tool access.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Permissions {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
}

impl Settings {
    /// Parse settings from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize settings to a pretty-printed JSON string.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_settings() {
        let json = r#"{
            "permissions": {
                "allow": ["Bash(cargo build)", "Bash(cargo test)"],
                "deny": []
            }
        }"#;
        let settings = Settings::from_json(json).unwrap();
        assert_eq!(settings.permissions.allow.len(), 2);
        assert!(settings.permissions.allow[0].contains("cargo build"));
    }

    #[test]
    fn test_empty_settings() {
        let json = "{}";
        let settings = Settings::from_json(json).unwrap();
        assert!(settings.permissions.allow.is_empty());
        assert!(settings.hooks.is_none());
    }
}
