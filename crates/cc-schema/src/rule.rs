use std::path::{Path, PathBuf};

/// A Claude Code rule resource.
///
/// Rules are plain Markdown files in `.claude/rules/`. No frontmatter —
/// the name is derived from the filename.
#[derive(Debug, Clone)]
pub struct Rule {
    /// Name derived from filename (without `.md` extension).
    pub name: String,
    /// Full Markdown content of the rule.
    pub body: String,
    /// Filesystem path where this rule was loaded from.
    pub source_path: PathBuf,
}

impl Rule {
    /// Parse a rule from file content and its path.
    pub fn parse(content: &str, path: &Path) -> Self {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Rule {
            name,
            body: content.to_string(),
            source_path: path.to_path_buf(),
        }
    }

    /// Derive the expected filename for this rule.
    pub fn filename(&self) -> String {
        format!("{}.md", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rule() {
        let path = PathBuf::from("python.md");
        let rule = Rule::parse("# Python Rules\nUse type hints always.", &path);
        assert_eq!(rule.name, "python");
        assert!(rule.body.contains("type hints"));
    }
}
