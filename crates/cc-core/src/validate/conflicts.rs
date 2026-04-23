use super::{ValidationFinding, ValidationLevel};
use crate::resource::ResourceEntry;
use cc_schema::ResourceType;
use std::collections::HashMap;

/// Check for resources with the same name across different origins.
pub fn check_conflicts(entries: &[ResourceEntry]) -> Vec<ValidationFinding> {
    let mut groups: HashMap<(ResourceType, String), Vec<&ResourceEntry>> = HashMap::new();

    for entry in entries {
        let key = (entry.resource_type, entry.name.clone());
        groups.entry(key).or_default().push(entry);
    }

    let mut findings = Vec::new();

    for ((rt, name), group) in groups {
        if group.len() > 1 {
            let origins: Vec<String> = group.iter().map(|e| e.origin.to_string()).collect();
            findings.push(ValidationFinding {
                level: ValidationLevel::Warning,
                resource_type: Some(rt),
                resource_name: Some(name),
                path: None,
                message: format!(
                    "exists in multiple origins: {} (highest precedence wins)",
                    origins.join(", ")
                ),
            });
        }
    }

    findings
}
