use super::ResourceEntry;
use cc_schema::ResourceType;
use std::collections::HashMap;

/// Resolve discovered resources by applying precedence rules.
///
/// When the same resource name exists in multiple origins, the highest-precedence
/// one becomes "active". Returns entries grouped by resource type.
pub fn resolve_resources(entries: &mut [ResourceEntry]) {
    // Group by (resource_type, name) and mark the highest-precedence as active
    let mut best: HashMap<(ResourceType, String), usize> = HashMap::new();

    for (i, entry) in entries.iter().enumerate() {
        let key = (entry.resource_type, entry.name.clone());
        match best.get(&key) {
            Some(&prev_idx) => {
                // Compare precedence: higher wins
                if entry.origin > entries[prev_idx].origin {
                    best.insert(key, i);
                }
            }
            None => {
                best.insert(key, i);
            }
        }
    }

    // Mark only the best entry for each (type, name) as active
    for (_, &idx) in best.iter() {
        entries[idx].active = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cc_schema::Origin;
    use std::path::PathBuf;

    #[test]
    fn test_resolve_single_origin() {
        let mut entries = vec![ResourceEntry {
            name: "commit".into(),
            resource_type: ResourceType::Skill,
            origin: Origin::Project,
            path: PathBuf::from(".claude/skills/commit.md"),
            active: false,
            description: Some("commit skill".into()),
            registry: None,
        }];
        resolve_resources(&mut entries);
        assert!(entries[0].active);
    }

    #[test]
    fn test_resolve_precedence() {
        let mut entries = vec![
            ResourceEntry {
                name: "react".into(),
                resource_type: ResourceType::Skill,
                origin: Origin::External {
                    library: "claude-skills".into(),
                },
                path: PathBuf::from("extern/claude-skills/skills/react.md"),
                active: false,
                description: None,
                registry: Some("extern/claude-skills".into()),
            },
            ResourceEntry {
                name: "react".into(),
                resource_type: ResourceType::Skill,
                origin: Origin::Project,
                path: PathBuf::from(".claude/skills/react.md"),
                active: false,
                description: None,
                registry: Some("project".into()),
            },
        ];
        resolve_resources(&mut entries);
        assert!(!entries[0].active); // External loses
        assert!(entries[1].active); // Project wins
    }
}
