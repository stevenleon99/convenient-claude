use crate::error::SchemaError;
use std::fs;
use std::path::Path;

/// Read a file to a string.
pub fn read_file(path: &Path) -> Result<String, SchemaError> {
    if !path.exists() {
        return Err(SchemaError::FileNotFound {
            path: path.to_path_buf(),
        });
    }
    fs::read_to_string(path).map_err(|source| SchemaError::Io { source })
}

/// Write content to a file, creating parent directories as needed.
pub fn write_file(path: &Path, content: &str) -> Result<(), SchemaError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| SchemaError::Io { source })?;
    }
    fs::write(path, content).map_err(|source| SchemaError::Io { source })
}

/// Read and parse a JSON file into a typed struct.
pub fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, SchemaError> {
    let content = read_file(path)?;
    serde_json::from_str(&content).map_err(|source| SchemaError::JsonError {
        path: path.to_path_buf(),
        source,
    })
}

/// Serialize and write a struct to a JSON file (pretty-printed).
pub fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), SchemaError> {
    let content = serde_json::to_string_pretty(value).map_err(|source| SchemaError::JsonError {
        path: path.to_path_buf(),
        source,
    })?;
    write_file(path, &content)
}

/// Append a line to a JSONL file.
pub fn append_jsonl<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), SchemaError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| SchemaError::Io { source })?;
    }
    let mut line = serde_json::to_string(value).map_err(|source| SchemaError::JsonError {
        path: path.to_path_buf(),
        source,
    })?;
    line.push('\n');

    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|source| SchemaError::Io { source })?;
    file.write_all(line.as_bytes())
        .map_err(|source| SchemaError::Io { source })
}

/// List all `.md` files in a directory (non-recursive).
pub fn list_md_files(dir: &Path) -> Result<Vec<std::path::PathBuf>, SchemaError> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    let entries = fs::read_dir(dir).map_err(|source| SchemaError::Io { source })?;
    for entry in entries {
        let entry = entry.map_err(|source| SchemaError::Io { source })?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

/// Recursively list all `.md` files in a directory.
pub fn list_md_files_recursive(dir: &Path) -> Result<Vec<std::path::PathBuf>, SchemaError> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    visit_md_files(dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn visit_md_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<(), SchemaError> {
    let entries = fs::read_dir(dir).map_err(|source| SchemaError::Io { source })?;
    for entry in entries {
        let entry = entry.map_err(|source| SchemaError::Io { source })?;
        let path = entry.path();
        if path.is_dir() {
            visit_md_files(&path, files)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_read_file_not_found() {
        let result = read_file(Path::new("/nonexistent/file.md"));
        assert!(result.is_err());
    }

    #[test]
    fn test_write_and_read_file() {
        let dir = std::env::temp_dir().join("cc-schema-test-write");
        let path = dir.join("test.md");
        write_file(&path, "hello").unwrap();
        let content = read_file(&path).unwrap();
        assert_eq!(content, "hello");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_write_and_read_json() {
        let dir = std::env::temp_dir().join("cc-schema-test-json");
        let path = dir.join("test.json");
        let data = serde_json::json!({"key": "value"});
        write_json(&path, &data).unwrap();
        let loaded: serde_json::Value = read_json(&path).unwrap();
        assert_eq!(loaded["key"], "value");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_list_md_files() {
        let dir = std::env::temp_dir().join("cc-schema-test-list");
        let _ = fs::create_dir_all(&dir);
        fs::write(dir.join("a.md"), "").unwrap();
        fs::write(dir.join("b.md"), "").unwrap();
        fs::write(dir.join("c.txt"), "").unwrap();

        let files = list_md_files(&dir).unwrap();
        assert_eq!(files.len(), 2);
        let _ = fs::remove_dir_all(&dir);
    }
}
