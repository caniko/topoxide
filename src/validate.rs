use std::collections::HashSet;

use thiserror::Error;

use crate::schema::GraphSpec;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("graph must define at least one group")]
    EmptyGroups,
    #[error("graph must define at least one node")]
    EmptyNodes,
    #[error("{entity} id must not be empty")]
    EmptyId { entity: &'static str },
    #[error("duplicate group id: {0}")]
    DuplicateGroup(String),
    #[error("duplicate node id: {0}")]
    DuplicateNode(String),
    #[error("duplicate edge: {edge_source} -> {target} ({kind})")]
    DuplicateEdge {
        edge_source: String,
        target: String,
        kind: String,
    },
    #[error("node {node} references unknown group {group}")]
    UnknownGroup { node: String, group: String },
    #[error("edge {edge_source} -> {target} references unknown source node")]
    UnknownSource { edge_source: String, target: String },
    #[error("edge {edge_source} -> {target} references unknown target node")]
    UnknownTarget { edge_source: String, target: String },
}

pub fn validate_graph(spec: &GraphSpec) -> Result<(), ValidationError> {
    if spec.groups.is_empty() {
        return Err(ValidationError::EmptyGroups);
    }
    if spec.nodes.is_empty() {
        return Err(ValidationError::EmptyNodes);
    }

    let mut groups = HashSet::new();
    for group in &spec.groups {
        if group.id.trim().is_empty() {
            return Err(ValidationError::EmptyId { entity: "group" });
        }
        if !groups.insert(group.id.as_str()) {
            return Err(ValidationError::DuplicateGroup(group.id.clone()));
        }
    }

    let mut nodes = HashSet::new();
    for node in &spec.nodes {
        if node.id.trim().is_empty() {
            return Err(ValidationError::EmptyId { entity: "node" });
        }
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

    let mut edges = HashSet::new();
    for edge in &spec.edges {
        if edge.source.trim().is_empty() || edge.target.trim().is_empty() {
            return Err(ValidationError::EmptyId { entity: "edge" });
        }
        let key = (
            edge.source.as_str(),
            edge.target.as_str(),
            edge.kind.as_str(),
            edge.label.as_deref(),
        );
        if !edges.insert(key) {
            return Err(ValidationError::DuplicateEdge {
                edge_source: edge.source.clone(),
                target: edge.target.clone(),
                kind: edge.kind.clone(),
            });
        }
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
