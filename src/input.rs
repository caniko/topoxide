use std::fs;
use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

use crate::schema::{GraphSpec, TopoxideDocument};

#[derive(Debug, Error)]
pub enum InputError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("unsupported input extension for {path}: {extension}")]
    UnsupportedExtension { path: String, extension: String },
    #[error("failed to parse JSON from {path}: {source}")]
    Json {
        path: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to parse TOML from {path}: {source}")]
    Toml {
        path: String,
        #[source]
        source: toml::de::Error,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum TopoxideInput {
    Document(TopoxideDocument),
    Graph(GraphSpec),
}

impl TopoxideInput {
    pub fn graph(&self) -> &GraphSpec {
        match self {
            Self::Document(document) => &document.graph,
            Self::Graph(graph) => graph,
        }
    }

    pub fn into_document(self) -> TopoxideDocument {
        match self {
            Self::Document(document) => document,
            Self::Graph(graph) => TopoxideDocument {
                graph,
                profiles: Vec::new(),
            },
        }
    }
}

pub fn read_input(path: impl AsRef<Path>) -> Result<TopoxideInput, InputError> {
    let path = path.as_ref();
    let path_display = path.display().to_string();
    let text = fs::read_to_string(path).map_err(|source| InputError::Read {
        path: path_display.clone(),
        source,
    })?;
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default();

    match extension {
        "json" => serde_json::from_str(&text).map_err(|source| InputError::Json {
            path: path_display,
            source,
        }),
        "toml" => toml::from_str(&text).map_err(|source| InputError::Toml {
            path: path_display,
            source,
        }),
        other => Err(InputError::UnsupportedExtension {
            path: path_display,
            extension: other.to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_path() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("simple.toml")
    }

    #[test]
    fn reads_toml_graph_fixture() {
        let input = read_input(fixture_path()).unwrap();
        assert_eq!(input.graph().title, "Simple graph");
        assert_eq!(input.graph().nodes.len(), 2);
    }

    #[test]
    fn rejects_unsupported_extension() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let error = read_input(tmp.path()).unwrap_err();
        assert!(matches!(error, InputError::UnsupportedExtension { .. }));
    }
}
