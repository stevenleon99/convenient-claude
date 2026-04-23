use std::path::PathBuf;

/// Errors originating from the data/schema layer.
#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    #[error("file not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("invalid frontmatter in {path}: {reason}")]
    InvalidFrontmatter { path: PathBuf, reason: String },

    #[error("missing required field '{field}' in {path}")]
    MissingField { path: PathBuf, field: String },

    #[error("JSON parse error in {path}: {source}")]
    JsonError {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("YAML parse error in {path}: {source}")]
    YamlError {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },

    #[error("I/O error: {source}")]
    Io {
        #[source]
        source: std::io::Error,
    },
}

impl From<std::io::Error> for SchemaError {
    fn from(source: std::io::Error) -> Self {
        SchemaError::Io { source }
    }
}
