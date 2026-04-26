use std::collections::{BTreeMap, BTreeSet};

use crate::schema::{
    EdgeMode, GraphEdge, GraphNode, GraphProfile, GraphSpec, MatrixColumn, ProfileViewKind,
    TopoxideDocument,
};
use crate::validate::{ValidationError, validate_graph};

#[derive(Debug, Clone)]
pub struct ProfileView {
    pub profile: GraphProfile,
    pub graph: GraphSpec,
    pub matrix: Option<MatrixView>,
}

#[derive(Debug, Clone)]
pub struct MatrixView {
    pub rows: Vec<MatrixRow>,
    pub columns: Vec<MatrixColumn>,
}

#[derive(Debug, Clone)]
pub struct MatrixRow {
    pub node: GraphNode,
    pub cells: Vec<MatrixCell>,
}

#[derive(Debug, Clone)]
pub struct MatrixCell {
    pub count: usize,
}

pub fn apply_profile(
    document: &TopoxideDocument,
    profile_id: Option<&str>,
) -> Result<ProfileView, ValidationError> {
    validate_graph(&document.graph)?;
    let profile = select_profile(document, profile_id);
    let graph = filtered_graph(&document.graph, &profile)?;
    let matrix = if profile.view == ProfileViewKind::Matrix {
        Some(build_matrix_view(&document.graph, &profile, &graph))
    } else {
        None
    };
    Ok(ProfileView {
        profile,
        graph,
        matrix,
    })
}

pub fn select_profile(document: &TopoxideDocument, profile_id: Option<&str>) -> GraphProfile {
    let selected = profile_id
        .and_then(|id| document.profiles.iter().find(|profile| profile.id == id))
        .or_else(|| document.profiles.first());
    selected.cloned().unwrap_or_else(|| GraphProfile {
        id: "default".to_owned(),
        label: "Default".to_owned(),
        description: None,
        view: ProfileViewKind::LaneGraph,
        edge_mode: EdgeMode::All,
        groups: Vec::new(),
        node_kinds: Vec::new(),
        edge_kinds: Vec::new(),
        show_lane_enclosures: true,
        matrix: None,
    })
}

fn filtered_graph(spec: &GraphSpec, profile: &GraphProfile) -> Result<GraphSpec, ValidationError> {
    let group_filter = to_set(&profile.groups);
    let kind_filter = to_set(&profile.node_kinds);
    let edge_filter = to_set(&profile.edge_kinds);

    let mut nodes = spec
        .nodes
        .iter()
        .filter(|node| {
            (group_filter.is_empty() || group_filter.contains(node.group.as_str()))
                && (kind_filter.is_empty() || kind_filter.contains(node.kind.as_str()))
        })
        .cloned()
        .collect::<Vec<_>>();
    let node_ids = nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<BTreeSet<_>>();

    let mut edges = if profile.edge_mode == EdgeMode::Hidden {
        Vec::new()
    } else {
        spec.edges
            .iter()
            .filter(|edge| {
                node_ids.contains(edge.source.as_str())
                    && node_ids.contains(edge.target.as_str())
                    && (edge_filter.is_empty() || edge_filter.contains(edge.kind.as_str()))
            })
            .cloned()
            .collect::<Vec<_>>()
    };

    if profile.edge_mode == EdgeMode::Bundled {
        edges = bundle_edges(spec, &nodes, &edges);
    }

    nodes.sort_by(|a, b| {
        a.group
            .cmp(&b.group)
            .then_with(|| a.order.cmp(&b.order))
            .then_with(|| a.label.cmp(&b.label))
            .then_with(|| a.id.cmp(&b.id))
    });

    let groups = spec
        .groups
        .iter()
        .filter(|group| nodes.iter().any(|node| node.group == group.id))
        .cloned()
        .collect();

    let graph = GraphSpec {
        title: spec.title.clone(),
        subtitle: profile
            .description
            .clone()
            .or_else(|| spec.subtitle.clone()),
        groups,
        nodes,
        edges,
        theme: spec.theme.clone(),
    };
    validate_graph(&graph)?;
    Ok(graph)
}

fn bundle_edges(spec: &GraphSpec, nodes: &[GraphNode], edges: &[GraphEdge]) -> Vec<GraphEdge> {
    let node_by_id = spec
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let visible_by_group =
        nodes
            .iter()
            .fold(BTreeMap::<&str, Vec<&GraphNode>>::new(), |mut acc, node| {
                acc.entry(node.group.as_str()).or_default().push(node);
                acc
            });
    let mut bundles = BTreeMap::<(String, String, String), usize>::new();
    for edge in edges {
        let Some(source) = node_by_id.get(edge.source.as_str()) else {
            continue;
        };
        let Some(target) = node_by_id.get(edge.target.as_str()) else {
            continue;
        };
        bundles
            .entry((
                source.group.clone(),
                target.group.clone(),
                edge.kind.clone(),
            ))
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }
    bundles
        .into_iter()
        .filter_map(|((source_group, target_group, kind), count)| {
            let source = visible_by_group.get(source_group.as_str())?.first()?;
            let target = visible_by_group.get(target_group.as_str())?.first()?;
            Some(GraphEdge {
                source: source.id.clone(),
                target: target.id.clone(),
                kind,
                label: Some(format!("{count} edge{}", if count == 1 { "" } else { "s" })),
            })
        })
        .collect()
}

fn build_matrix_view(
    source: &GraphSpec,
    profile: &GraphProfile,
    filtered: &GraphSpec,
) -> MatrixView {
    let Some(config) = &profile.matrix else {
        return MatrixView {
            rows: Vec::new(),
            columns: Vec::new(),
        };
    };
    let nodes = source
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let mut rows = filtered
        .nodes
        .iter()
        .filter(|node| node.group == config.row_group)
        .cloned()
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| a.order.cmp(&b.order).then_with(|| a.label.cmp(&b.label)));

    let rows = rows
        .into_iter()
        .map(|node| {
            let cells = config
                .columns
                .iter()
                .map(|column| {
                    let count = source
                        .edges
                        .iter()
                        .filter(|edge| {
                            edge.source == node.id && column_matches_edge(column, edge, &nodes)
                        })
                        .count();
                    MatrixCell { count }
                })
                .collect();
            MatrixRow { node, cells }
        })
        .collect();
    MatrixView {
        rows,
        columns: config.columns.clone(),
    }
}

fn column_matches_edge(
    column: &MatrixColumn,
    edge: &GraphEdge,
    nodes: &BTreeMap<&str, &GraphNode>,
) -> bool {
    if !column.edge_kinds.is_empty() && !column.edge_kinds.iter().any(|kind| kind == &edge.kind) {
        return false;
    }
    let Some(target) = nodes.get(edge.target.as_str()) else {
        return false;
    };
    (column.target_groups.is_empty()
        || column
            .target_groups
            .iter()
            .any(|group| group == &target.group))
        && (column.target_kinds.is_empty()
            || column.target_kinds.iter().any(|kind| kind == &target.kind))
}

fn to_set(values: &[String]) -> BTreeSet<&str> {
    values.iter().map(String::as_str).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{GraphGroup, GraphTheme};

    #[test]
    fn profile_bundles_edges_by_group_pair_and_kind() {
        let document = fixture_document();
        let view = apply_profile(&document, Some("overview")).unwrap();
        assert_eq!(view.graph.edges.len(), 1);
        assert_eq!(view.graph.edges[0].label.as_deref(), Some("2 edges"));
    }

    #[test]
    fn matrix_profile_counts_matching_edges() {
        let document = fixture_document();
        let view = apply_profile(&document, Some("matrix")).unwrap();
        let matrix = view.matrix.unwrap();
        assert_eq!(matrix.rows.len(), 1);
        assert_eq!(matrix.rows[0].cells[0].count, 2);
    }

    fn fixture_document() -> TopoxideDocument {
        let graph = GraphSpec {
            title: "fixture".into(),
            subtitle: None,
            groups: vec![
                GraphGroup {
                    id: "a".into(),
                    label: "A".into(),
                    order: 0,
                    description: None,
                },
                GraphGroup {
                    id: "b".into(),
                    label: "B".into(),
                    order: 1,
                    description: None,
                },
            ],
            nodes: vec![
                GraphNode {
                    id: "a1".into(),
                    label: "A1".into(),
                    kind: "source".into(),
                    group: "a".into(),
                    detail: None,
                    metrics: Vec::new(),
                    order: 0,
                    weight: None,
                },
                GraphNode {
                    id: "b1".into(),
                    label: "B1".into(),
                    kind: "target".into(),
                    group: "b".into(),
                    detail: None,
                    metrics: Vec::new(),
                    order: 0,
                    weight: None,
                },
                GraphNode {
                    id: "b2".into(),
                    label: "B2".into(),
                    kind: "target".into(),
                    group: "b".into(),
                    detail: None,
                    metrics: Vec::new(),
                    order: 1,
                    weight: None,
                },
            ],
            edges: vec![
                GraphEdge {
                    source: "a1".into(),
                    target: "b1".into(),
                    kind: "feeds".into(),
                    label: None,
                },
                GraphEdge {
                    source: "a1".into(),
                    target: "b2".into(),
                    kind: "feeds".into(),
                    label: None,
                },
            ],
            theme: GraphTheme::default(),
        };
        TopoxideDocument {
            graph,
            profiles: vec![
                GraphProfile {
                    id: "overview".into(),
                    label: "Overview".into(),
                    description: None,
                    view: ProfileViewKind::LaneGraph,
                    edge_mode: EdgeMode::Bundled,
                    groups: vec!["a".into(), "b".into()],
                    node_kinds: Vec::new(),
                    edge_kinds: vec!["feeds".into()],
                    show_lane_enclosures: true,
                    matrix: None,
                },
                GraphProfile {
                    id: "matrix".into(),
                    label: "Matrix".into(),
                    description: None,
                    view: ProfileViewKind::Matrix,
                    edge_mode: EdgeMode::Hidden,
                    groups: vec!["a".into()],
                    node_kinds: Vec::new(),
                    edge_kinds: Vec::new(),
                    show_lane_enclosures: true,
                    matrix: Some(crate::schema::MatrixProfile {
                        row_group: "a".into(),
                        columns: vec![MatrixColumn {
                            id: "feeds".into(),
                            label: "Feeds".into(),
                            edge_kinds: vec!["feeds".into()],
                            target_groups: vec!["b".into()],
                            target_kinds: Vec::new(),
                        }],
                    }),
                },
            ],
        }
    }
}
