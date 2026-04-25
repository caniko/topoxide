use std::collections::HashMap;

use crate::schema::{GraphNode, GraphSpec};
use crate::validate::{validate_graph, ValidationError};

#[derive(Debug, Clone, Copy)]
pub struct RenderOptions {
    pub width: f32,
    pub height: f32,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: 1800.0,
            height: 1200.0,
        }
    }
}

#[derive(Debug, Clone)]
struct PlacedNode<'a> {
    node: &'a GraphNode,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

pub fn render_svg(spec: &GraphSpec, options: RenderOptions) -> Result<String, ValidationError> {
    validate_graph(spec)?;

    let margin = 56.0;
    let title_h = 82.0;
    let lane_header_h = 42.0;
    let group_gap = 18.0;
    let node_w: f32 = 188.0;
    let node_h: f32 = 56.0;
    let node_gap: f32 = 18.0;

    let mut groups = spec.groups.iter().collect::<Vec<_>>();
    groups.sort_by(|a, b| {
        a.order
            .cmp(&b.order)
            .then_with(|| a.label.cmp(&b.label))
            .then_with(|| a.id.cmp(&b.id))
    });

    let lane_count = groups.len().max(1);
    let lane_w = (options.width - margin * 2.0 - group_gap * (lane_count.saturating_sub(1) as f32))
        / lane_count as f32;
    let top = margin + title_h;
    let lane_h = options.height - top - margin;

    let mut by_group: HashMap<&str, Vec<&GraphNode>> = HashMap::new();
    for node in &spec.nodes {
        by_group.entry(&node.group).or_default().push(node);
    }
    for nodes in by_group.values_mut() {
        nodes.sort_by(|a, b| {
            a.order
                .cmp(&b.order)
                .then_with(|| a.kind.cmp(&b.kind))
                .then_with(|| a.label.cmp(&b.label))
                .then_with(|| a.id.cmp(&b.id))
        });
    }

    let mut placed = HashMap::new();
    for (group_idx, group) in groups.iter().enumerate() {
        let lane_x = margin + group_idx as f32 * (lane_w + group_gap);
        let nodes = by_group.get(group.id.as_str()).cloned().unwrap_or_default();
        let count = nodes.len().max(1);
        let available_h = lane_h - lane_header_h - 28.0;
        let step = if count <= 1 {
            0.0
        } else {
            ((available_h - node_h) / (count - 1) as f32).max(node_h + node_gap)
        };
        let actual_h = node_h + step * (count.saturating_sub(1) as f32);
        let start_y = top + lane_header_h + ((available_h - actual_h).max(0.0) / 2.0) + 14.0;
        for (node_idx, node) in nodes.iter().enumerate() {
            let w = node_w.min(lane_w - 22.0).max(112.0);
            let x = lane_x + (lane_w - w) / 2.0;
            let y = start_y + node_idx as f32 * step;
            placed.insert(
                node.id.as_str(),
                PlacedNode {
                    node,
                    x,
                    y,
                    w,
                    h: node_h,
                },
            );
        }
    }

    let mut svg = String::new();
    push_line(
        &mut svg,
        &format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{:.0}" height="{:.0}" viewBox="0 0 {:.0} {:.0}" role="img" aria-label="{}">"##,
            options.width,
            options.height,
            options.width,
            options.height,
            esc_attr(&spec.title)
        ),
    );
    push_line(&mut svg, "<defs>");
    push_line(
        &mut svg,
        &format!(
            r##"<marker id="arrow" viewBox="0 0 10 10" refX="8.5" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" fill="{}"/></marker>"##,
            spec.theme.muted
        ),
    );
    push_line(
        &mut svg,
        r##"<filter id="soft-shadow" x="-10%" y="-10%" width="120%" height="130%"><feDropShadow dx="0" dy="1.5" stdDeviation="1.8" flood-color="#000000" flood-opacity="0.10"/></filter>"##,
    );
    push_line(&mut svg, "</defs>");
    push_line(
        &mut svg,
        &format!(
            r##"<rect width="100%" height="100%" fill="{}"/>"##,
            spec.theme.background
        ),
    );
    push_line(
        &mut svg,
        &format!(
            r##"<text x="{margin}" y="48" fill="{}" font-family="{}" font-size="28" font-weight="600">{}</text>"##,
            spec.theme.ink,
            esc_attr(&spec.theme.font_family),
            esc_text(&spec.title)
        ),
    );
    if let Some(subtitle) = &spec.subtitle {
        push_line(
            &mut svg,
            &format!(
                r##"<text x="{margin}" y="76" fill="{}" font-family="{}" font-size="15">{}</text>"##,
                spec.theme.muted,
                esc_attr(&spec.theme.font_family),
                esc_text(subtitle)
            ),
        );
    }

    for (group_idx, group) in groups.iter().enumerate() {
        let lane_x = margin + group_idx as f32 * (lane_w + group_gap);
        push_line(
            &mut svg,
            &format!(
                r##"<rect x="{lane_x:.1}" y="{top:.1}" width="{lane_w:.1}" height="{lane_h:.1}" rx="8" fill="#ffffff" stroke="{}" stroke-width="1"/>"##,
                spec.theme.grid
            ),
        );
        push_line(
            &mut svg,
            &format!(
                r##"<text x="{:.1}" y="{:.1}" fill="{}" font-family="{}" font-size="14" font-weight="700">{}</text>"##,
                lane_x + 14.0,
                top + 26.0,
                spec.theme.ink,
                esc_attr(&spec.theme.font_family),
                esc_text(&group.label)
            ),
        );
    }

    let mut edges = spec.edges.iter().collect::<Vec<_>>();
    edges.sort_by(|a, b| {
        a.source
            .cmp(&b.source)
            .then_with(|| a.target.cmp(&b.target))
            .then_with(|| a.kind.cmp(&b.kind))
    });
    for edge in edges {
        let source = &placed[edge.source.as_str()];
        let target = &placed[edge.target.as_str()];
        let sx = source.x + source.w;
        let sy = source.y + source.h / 2.0;
        let tx = target.x;
        let ty = target.y + target.h / 2.0;
        let dx = (tx - sx).abs().max(38.0);
        let c1 = sx + dx * 0.45;
        let c2 = tx - dx * 0.45;
        push_line(
            &mut svg,
            &format!(
                r##"<path d="M {sx:.1} {sy:.1} C {c1:.1} {sy:.1}, {c2:.1} {ty:.1}, {tx:.1} {ty:.1}" fill="none" stroke="{}" stroke-width="1.2" marker-end="url(#arrow)" opacity="0.72"/>"##,
                spec.theme.muted
            ),
        );
    }

    let mut placed_nodes = placed.values().collect::<Vec<_>>();
    placed_nodes.sort_by(|a, b| {
        a.x.partial_cmp(&b.x)
            .unwrap()
            .then_with(|| a.y.partial_cmp(&b.y).unwrap())
            .then_with(|| a.node.id.cmp(&b.node.id))
    });
    for node in placed_nodes {
        let color = kind_color(&node.node.kind, &spec.theme.accent);
        push_line(
            &mut svg,
            &format!(
                r##"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="7" fill="#ffffff" stroke="{color}" stroke-width="1.4" filter="url(#soft-shadow)"/>"##,
                node.x, node.y, node.w, node.h
            ),
        );
        push_line(
            &mut svg,
            &format!(
                r##"<rect x="{:.1}" y="{:.1}" width="5" height="{:.1}" rx="2.5" fill="{color}"/>"##,
                node.x + 8.0,
                node.y + 9.0,
                node.h - 18.0
            ),
        );
        push_wrapped_text(
            &mut svg,
            &node.node.label,
            node.x + 22.0,
            node.y + 23.0,
            node.w - 32.0,
            13.0,
            2,
            &spec.theme.ink,
            &spec.theme.font_family,
            true,
        );
        if let Some(detail) = &node.node.detail {
            push_wrapped_text(
                &mut svg,
                detail,
                node.x + 22.0,
                node.y + 44.0,
                node.w - 32.0,
                10.5,
                1,
                &spec.theme.muted,
                &spec.theme.font_family,
                false,
            );
        }
    }

    push_legend(&mut svg, spec, options);
    push_line(&mut svg, "</svg>");
    Ok(svg)
}

fn push_legend(svg: &mut String, spec: &GraphSpec, options: RenderOptions) {
    let kinds = {
        let mut kinds = spec
            .nodes
            .iter()
            .map(|n| n.kind.as_str())
            .collect::<Vec<_>>();
        kinds.sort_unstable();
        kinds.dedup();
        kinds
    };
    let x = 58.0;
    let y = options.height - 24.0;
    let mut cursor = x;
    for kind in kinds {
        let color = kind_color(kind, &spec.theme.accent);
        push_line(
            svg,
            &format!(
                r##"<circle cx="{cursor:.1}" cy="{y:.1}" r="4" fill="{color}"/><text x="{:.1}" y="{:.1}" fill="{}" font-family="{}" font-size="10">{}</text>"##,
                cursor + 9.0,
                y + 3.5,
                spec.theme.muted,
                esc_attr(&spec.theme.font_family),
                esc_text(kind)
            ),
        );
        cursor += (kind.len() as f32 * 6.2) + 34.0;
    }
}

fn kind_color(kind: &str, fallback: &str) -> String {
    match kind {
        "source-family" => "#009E73",
        "dataset" => "#56B4E9",
        "etl-job" | "etl-stage" => "#0F1689",
        "object-storage" | "s3-artifact" => "#D55E00",
        "clickhouse-table" => "#E6AC00",
        "materialized-view" => "#CC79A7",
        "precomputed-table" => "#0072B2",
        "consumer" => "#111827",
        _ => fallback,
    }
    .to_owned()
}

fn push_wrapped_text(
    svg: &mut String,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    font_size: f32,
    max_lines: usize,
    fill: &str,
    font_family: &str,
    bold: bool,
) {
    let max_chars = (width / (font_size * 0.54)).floor().max(8.0) as usize;
    let lines = wrap_text(text, max_chars, max_lines);
    let weight = if bold { r##" font-weight="600""## } else { "" };
    push_line(
        svg,
        &format!(
            r##"<text x="{x:.1}" y="{y:.1}" fill="{fill}" font-family="{}" font-size="{font_size:.1}"{weight}>"##,
            esc_attr(font_family)
        ),
    );
    for (idx, line) in lines.iter().enumerate() {
        let dy = if idx == 0 {
            0.0
        } else {
            font_size * 1.15 * idx as f32
        };
        push_line(
            svg,
            &format!(
                r##"<tspan x="{x:.1}" dy="{dy:.1}">{}</tspan>"##,
                esc_text(line)
            ),
        );
    }
    push_line(svg, "</text>");
}

fn wrap_text(text: &str, max_chars: usize, max_lines: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let sep = usize::from(!current.is_empty());
        if current.len() + sep + word.len() > max_chars && !current.is_empty() {
            lines.push(current);
            current = String::new();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.len() > max_lines {
        lines.truncate(max_lines);
        if let Some(last) = lines.last_mut() {
            if last.len() > 3 {
                last.truncate(last.len().saturating_sub(3));
            }
            last.push_str("...");
        }
    }
    lines
}

fn esc_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn esc_attr(s: &str) -> String {
    esc_text(s).replace('"', "&quot;")
}

fn push_line(svg: &mut String, line: &str) {
    svg.push_str(line);
    svg.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{GraphEdge, GraphGroup, GraphNode, GraphSpec, GraphTheme};

    #[test]
    fn render_is_deterministic() {
        let spec = GraphSpec {
            title: "Test".into(),
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
                    id: "n2".into(),
                    label: "Target".into(),
                    kind: "clickhouse-table".into(),
                    group: "b".into(),
                    detail: None,
                    order: 0,
                    weight: None,
                },
                GraphNode {
                    id: "n1".into(),
                    label: "Source".into(),
                    kind: "dataset".into(),
                    group: "a".into(),
                    detail: None,
                    order: 0,
                    weight: None,
                },
            ],
            edges: vec![GraphEdge {
                source: "n1".into(),
                target: "n2".into(),
                kind: "feeds".into(),
                label: None,
            }],
            theme: GraphTheme::default(),
        };

        let a = render_svg(&spec, RenderOptions::default()).unwrap();
        let b = render_svg(&spec, RenderOptions::default()).unwrap();
        assert_eq!(a, b);
        assert!(a.contains("<svg"));
        assert!(a.contains("marker-end=\"url(#arrow)\""));
    }
}
