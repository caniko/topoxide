use std::collections::HashSet;

use thiserror::Error;

use crate::schema::GraphSpec;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("duplicate group id: {0}")]
    DuplicateGroup(String),
    #[error("duplicate node id: {0}")]
    DuplicateNode(String),
    #[error("node {node} references unknown group {group}")]
    UnknownGroup { node: String, group: String },
    #[error("edge {edge_source} -> {target} references unknown source node")]
    UnknownSource { edge_source: String, target: String },
    #[error("edge {edge_source} -> {target} references unknown target node")]
    UnknownTarget { edge_source: String, target: String },
}

pub fn validate_graph(spec: &GraphSpec) -> Result<(), ValidationError> {
    let mut groups = HashSet::new();
    for group in &spec.groups {
        if !groups.insert(group.id.as_str()) {
            return Err(ValidationError::DuplicateGroup(group.id.clone()));
        }
    }

    let mut nodes = HashSet::new();
    for node in &spec.nodes {
        if !nodes.insert(node.id.as_str()) {
            return Err(ValidationError::DuplicateNode(node.id.clone()));
        }
        if !groups.contains(node.group.as_str()) {
            return Err(ValidationError::UnknownGroup {
                node: node.id.clone(),
                group: node.group.clone(),
            });
        }
    }

    for edge in &spec.edges {
        if !nodes.contains(edge.source.as_str()) {
            return Err(ValidationError::UnknownSource {
                edge_source: edge.source.clone(),
                target: edge.target.clone(),
            });
        }
        if !nodes.contains(edge.target.as_str()) {
            return Err(ValidationError::UnknownTarget {
                edge_source: edge.source.clone(),
                target: edge.target.clone(),
            });
        }
    }

    Ok(())
}
