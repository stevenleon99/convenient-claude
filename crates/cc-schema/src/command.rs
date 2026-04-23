use crate::error::SchemaError;
use crate::frontmatter::parse_frontmatter;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A Claude Code command resource.
///
/// Commands are Markdown files with YAML frontmatter, stored in `.claude/commands/`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    /// The Markdown body after the frontmatter.
    #[serde(skip)]
    pub body: String,
    /// The file path this command was loaded from.
    #[serde(skip)]
    pub source_path: Option<std::path::PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CommandFrontmatter {
    name: String,
    description: String,
    #[serde(default)]
    allowed_tools: Vec<String>,
}

impl Command {
    /// Parse a command from raw Markdown content with YAML frontmatter.
    pub fn parse(content: &str, path: &Path) -> Result<Self, SchemaError> {
        let (fm, body) = parse_frontmatter::<CommandFrontmatter>(content, path)?;

        Ok(Command {
            name: fm.name,
            description: fm.description,
            allowed_tools: fm.allowed_tools,
            body: body.to_string(),
            source_path: Some(path.to_path_buf()),
        })
    }

    /// Derive the expected filename for this command.
    pub fn filename(&self) -> String {
        format!("{}.md", self.name)
    }

    /// Serialize this command back to Markdown with YAML frontmatter.
    pub fn to_markdown(&self) -> Result<String, SchemaError> {
        let frontmatter = CommandFrontmatter {
            name: self.name.clone(),
            description: self.description.clone(),
            allowed_tools: self.allowed_tools.clone(),
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
    fn test_parse_command() {
        let content = "\
---
name: deploy
description: Deploy to production
allowed_tools: [Bash, Read]
---

# /deploy

Deploy the application.
";
        let cmd = Command::parse(content, Path::new("deploy.md")).unwrap();
        assert_eq!(cmd.name, "deploy");
        assert_eq!(cmd.description, "Deploy to production");
        assert_eq!(cmd.allowed_tools, vec!["Bash", "Read"]);
        assert!(cmd.body.contains("/deploy"));
    }

    #[test]
    fn test_command_no_allowed_tools() {
        let content = "\
---
name: status
description: Check status
---

# /status
";
        let cmd = Command::parse(content, Path::new("status.md")).unwrap();
        assert!(cmd.allowed_tools.is_empty());
    }
}
