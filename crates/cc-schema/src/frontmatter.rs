use crate::error::SchemaError;
use std::path::Path;

/// Extract YAML frontmatter (between `---` delimiters) and body from a Markdown string.
///
/// Returns `(frontmatter_str, body_str)`.
/// If no frontmatter is found, returns `("", full_content)`.
pub fn extract_frontmatter(content: &str) -> (&str, &str) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return ("", content);
    }

    // Skip opening ---
    let after_open = &trimmed[3..];
    // Find closing ---
    match after_open.find("\n---") {
        Some(end) => {
            let frontmatter = after_open[..end].trim();
            let body_start = end + 4; // skip \n---
            let body = after_open[body_start..].trim();
            (frontmatter, body)
        }
        None => ("", content),
    }
}

/// Parse YAML frontmatter from a file, returning deserialized T and the body.
pub fn parse_frontmatter<'de, T: serde::Deserialize<'de>>(
    content: &'de str,
    path: &Path,
) -> Result<(T, &'de str), SchemaError> {
    let (frontmatter, body) = extract_frontmatter(content);

    if frontmatter.is_empty() {
        return Err(SchemaError::InvalidFrontmatter {
            path: path.to_path_buf(),
            reason: "no YAML frontmatter found (expected --- delimiters)".to_string(),
        });
    }

    let parsed: T = serde_yaml::from_str(frontmatter).map_err(|e| SchemaError::YamlError {
        path: path.to_path_buf(),
        source: e,
    })?;

    Ok((parsed, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_frontmatter_present() {
        let content = "---\nname: test\n---\n# Body content\n";
        let (fm, body) = extract_frontmatter(content);
        assert_eq!(fm, "name: test");
        assert_eq!(body, "# Body content");
    }

    #[test]
    fn test_extract_frontmatter_absent() {
        let content = "# Just markdown\nNo frontmatter here";
        let (fm, body) = extract_frontmatter(content);
        assert_eq!(fm, "");
        assert!(body.contains("Just markdown"));
    }

    #[test]
    fn test_extract_frontmatter_unclosed() {
        let content = "---\nname: test\nno closing delimiter";
        let (fm, body) = extract_frontmatter(content);
        assert_eq!(fm, "");
        assert!(!body.is_empty());
    }

    #[test]
    fn test_parse_frontmatter_valid() {
        #[derive(serde::Deserialize)]
        struct TestData {
            name: String,
        }

        let content = "---\nname: hello\n---\nBody";
        let (data, body) = parse_frontmatter::<TestData>(content, Path::new("test.md")).unwrap();
        assert_eq!(data.name, "hello");
        assert_eq!(body, "Body");
    }

    #[test]
    fn test_parse_frontmatter_invalid_yaml() {
        #[derive(serde::Deserialize)]
        #[allow(dead_code)]
        struct TestData {
            name: String,
        }

        let content = "---\n: invalid\n---\nBody";
        let result = parse_frontmatter::<TestData>(content, Path::new("test.md"));
        assert!(result.is_err());
    }
}
