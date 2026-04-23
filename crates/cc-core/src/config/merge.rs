use cc_schema::{Permissions, Settings};

/// Merge user and project settings. Project values override user values.
pub fn merge_settings(user: &Settings, project: &Settings) -> Settings {
    Settings {
        permissions: merge_permissions(&user.permissions, &project.permissions),
        hooks: project.hooks.clone().or_else(|| user.hooks.clone()),
    }
}

fn merge_permissions(user: &Permissions, project: &Permissions) -> Permissions {
    // Start with user permissions, then overlay project permissions
    let mut allow = user.allow.clone();
    for entry in &project.allow {
        if !allow.contains(entry) {
            allow.push(entry.clone());
        }
    }

    let mut deny = user.deny.clone();
    for entry in &project.deny {
        if !deny.contains(entry) {
            deny.push(entry.clone());
        }
    }

    Permissions { allow, deny }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_permissions() {
        let user = Permissions {
            allow: vec!["Bash(cargo build)".into(), "Bash(cargo test)".into()],
            deny: vec!["Bash(rm *)".into()],
        };
        let project = Permissions {
            allow: vec!["Bash(cargo clippy)".into()],
            deny: vec![],
        };

        let merged = merge_permissions(&user, &project);
        assert_eq!(merged.allow.len(), 3);
        assert!(merged.allow.contains(&"Bash(cargo build)".to_string()));
        assert!(merged.allow.contains(&"Bash(cargo clippy)".to_string()));
        assert_eq!(merged.deny.len(), 1);
    }

    #[test]
    fn test_merge_hooks_project_overrides() {
        let user = Settings {
            permissions: Permissions::default(),
            hooks: Some(cc_schema::HookConfig::default()),
        };
        let project = Settings {
            permissions: Permissions::default(),
            hooks: None,
        };

        let merged = merge_settings(&user, &project);
        // Project has no hooks, so user hooks should be inherited
        assert!(merged.hooks.is_some());
    }
}
