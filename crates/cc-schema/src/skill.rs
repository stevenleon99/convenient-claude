use crate::error::SchemaError;
use crate::frontmatter::parse_frontmatter;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A Claude Code skill resource.
///
/// Skills are Markdown files with YAML frontmatter, stored in `.claude/skills/`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(default)]
    pub metadata: SkillMetadata,
    /// The Markdown body after the frontmatter.
    #[serde(skip)]
    pub body: String,
    /// The file path this skill was loaded from.
    #[serde(skip)]
    pub source_path: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub triggers: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "output-format"
    )]
    pub output_format: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "related-skills"
    )]
    pub related_skills: Option<String>,
}

/// Intermediate struct for deserializing just the frontmatter fields.
#[derive(Debug, Serialize, Deserialize)]
struct SkillFrontmatter {
    name: String,
    description: String,
    #[serde(default)]
    license: Option<String>,
    #[serde(default)]
    metadata: SkillMetadata,
}

impl Skill {
    /// Parse a skill from raw Markdown content with YAML frontmatter.
    pub fn parse(content: &str, path: &Path) -> Result<Self, SchemaError> {
        let (fm, body) = parse_frontmatter::<SkillFrontmatter>(content, path)?;

        Ok(Skill {
            name: fm.name,
            description: fm.description,
            license: fm.license,
            metadata: fm.metadata,
            body: body.to_string(),
            source_path: Some(path.to_path_buf()),
        })
    }

    /// Derive the expected filename for this skill.
    pub fn filename(&self) -> String {
        format!("{}.md", self.name)
    }

    /// Serialize this skill back to Markdown with YAML frontmatter.
    pub fn to_markdown(&self) -> Result<String, SchemaError> {
        let frontmatter = SkillFrontmatter {
            name: self.name.clone(),
            description: self.description.clone(),
            license: self.license.clone(),
            metadata: self.metadata.clone(),
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
    fn test_parse_skill() {
        let content = "\
---
name: react-expert
description: React/Next.js specialist
license: MIT
metadata:
  author: Jeffallan
  version: \"1.0.0\"
  domain: frontend
  triggers: react, nextjs, component
  role: specialist
---

# React Expert

You are a React expert.
";
        let skill = Skill::parse(content, Path::new("react-expert.md")).unwrap();
        assert_eq!(skill.name, "react-expert");
        assert_eq!(skill.description, "React/Next.js specialist");
        assert_eq!(skill.license.as_deref(), Some("MIT"));
        assert_eq!(skill.metadata.author.as_deref(), Some("Jeffallan"));
        assert_eq!(skill.metadata.domain.as_deref(), Some("frontend"));
        assert!(skill.body.contains("React Expert"));
    }

    #[test]
    fn test_skill_roundtrip() {
        let content = "\
---
name: test
description: A test skill
---

Body here.
";
        let skill = Skill::parse(content, Path::new("test.md")).unwrap();
        let md = skill.to_markdown().unwrap();
        let reparsed = Skill::parse(&md, Path::new("test.md")).unwrap();
        assert_eq!(skill.name, reparsed.name);
        assert_eq!(skill.description, reparsed.description);
    }

    #[test]
    fn test_skill_missing_name() {
        let content = "\
---
description: No name field
---

Body.
";
        let result = Skill::parse(content, Path::new("bad.md"));
        assert!(result.is_err());
    }
}
