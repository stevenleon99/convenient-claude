use crate::error::SchemaError;
use crate::frontmatter::parse_frontmatter;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A Claude Code agent resource.
///
/// Agents are Markdown files with YAML frontmatter, stored in `.claude/agents/`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub name: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default)]
    pub tools: Vec<String>,
    /// The Markdown body after the frontmatter.
    #[serde(skip)]
    pub body: String,
    /// The file path this agent was loaded from.
    #[serde(skip)]
    pub source_path: Option<std::path::PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AgentFrontmatter {
    name: String,
    description: String,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    tools: Vec<String>,
}

impl Agent {
    /// Parse an agent from raw Markdown content with YAML frontmatter.
    pub fn parse(content: &str, path: &Path) -> Result<Self, SchemaError> {
        let (fm, body) = parse_frontmatter::<AgentFrontmatter>(content, path)?;

        Ok(Agent {
            name: fm.name,
            description: fm.description,
            model: fm.model,
            tools: fm.tools,
            body: body.to_string(),
            source_path: Some(path.to_path_buf()),
        })
    }

    /// Derive the expected filename for this agent.
    pub fn filename(&self) -> String {
        format!("{}.md", self.name)
    }

    /// Serialize this agent back to Markdown with YAML frontmatter.
    pub fn to_markdown(&self) -> Result<String, SchemaError> {
        let frontmatter = AgentFrontmatter {
            name: self.name.clone(),
            description: self.description.clone(),
            model: self.model.clone(),
            tools: self.tools.clone(),
        };
        let yaml = serde_yaml::to_string(&frontmatter).map_err(|e| SchemaError::YamlError {
            path: self.source_path.clone().unwrap_or_default(),
            source: e,
        })?;
        Ok(format!("---\n{}---\n\n{}\n", yaml, self.body))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_agent() {
        let content = "\
---
name: docs-researcher
description: Research agent for documentation
model: sonnet
tools: [Bash, Read, Grep, Glob]
---

# Docs Researcher

You research documentation.
";
        let agent = Agent::parse(content, Path::new("docs-researcher.md")).unwrap();
        assert_eq!(agent.name, "docs-researcher");
        assert_eq!(agent.model.as_deref(), Some("sonnet"));
        assert_eq!(agent.tools, vec!["Bash", "Read", "Grep", "Glob"]);
    }
}
