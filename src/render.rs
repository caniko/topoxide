use std::collections::{BTreeMap, HashMap};

use crate::profile::{MatrixView, ProfileView, apply_profile};
use crate::schema::{GraphEdge, GraphNode, GraphSpec, ProfileViewKind, TopoxideDocument};
use crate::validate::{ValidationError, validate_graph};

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
    render_lane_graph_svg(spec, options, true)
}

fn render_lane_graph_svg(
    spec: &GraphSpec,
    options: RenderOptions,
    show_lane_enclosures: bool,
) -> Result<String, ValidationError> {
    validate_graph(spec)?;

    let margin = 56.0;
    let title_h = 82.0;
    let group_gap = 18.0;
    let node_w: f32 = 188.0;

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
    let show_lane_headers = should_show_lane_headers(&groups, &by_group);
    let lane_header_h = if show_lane_headers { 42.0 } else { 10.0 };

    let mut placed = HashMap::new();
    for (group_idx, group) in groups.iter().enumerate() {
        let lane_x = margin + group_idx as f32 * (lane_w + group_gap);
        let nodes = by_group.get(group.id.as_str()).cloned().unwrap_or_default();
        let count = nodes.len().max(1);
        let available_h = lane_h - lane_header_h - 28.0;
        let node_h = adaptive_node_height_for_nodes(available_h, &nodes);
        let step = adaptive_step(available_h, count, node_h);
        let actual_h = node_h + step * (count.saturating_sub(1) as f32);
        let start_y = top + lane_header_h + ((available_h - actual_h).max(0.0) / 2.0) + 14.0;
        for (node_idx, node) in nodes.iter().enumerate() {
            let desired_w = if node.metrics.is_empty() {
                node_w
            } else {
                lane_w - 18.0
            };
            let w = desired_w.min(lane_w - 22.0).max(112.0);
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
        if show_lane_enclosures {
            push_line(
                &mut svg,
                &format!(
                    r##"<rect class="lane-enclosure" x="{lane_x:.1}" y="{top:.1}" width="{lane_w:.1}" height="{lane_h:.1}" rx="8" fill="#ffffff" stroke="{}" stroke-width="1"/>"##,
                    spec.theme.grid
                ),
            );
        }
        if show_lane_headers {
            push_line(
                &mut svg,
                &format!(
                    r##"<text class="lane-header" x="{:.1}" y="{:.1}" fill="{}" font-family="{}" font-size="14" font-weight="700">{}</text>"##,
                    lane_x + 14.0,
                    top + 26.0,
                    spec.theme.ink,
                    esc_attr(&spec.theme.font_family),
                    esc_text(&group.label)
                ),
            );
        }
    }

    let mut edges = spec.edges.iter().collect::<Vec<_>>();
    edges.sort_by(|a, b| {
        a.source
            .cmp(&b.source)
            .then_with(|| a.target.cmp(&b.target))
            .then_with(|| a.kind.cmp(&b.kind))
    });
    render_overview_detail_edges(&mut svg, &edges, &placed, spec);
    for edge in edges
        .into_iter()
        .filter(|edge| edge.kind != "overview-detail")
    {
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
        if let Some(label) = &edge.label {
            let mx = (sx + tx) / 2.0;
            let my = (sy + ty) / 2.0 - 4.0;
            push_line(
                &mut svg,
                &format!(
                    r##"<text x="{mx:.1}" y="{my:.1}" fill="{}" font-family="{}" font-size="9" text-anchor="middle">{}</text>"##,
                    spec.theme.muted,
                    esc_attr(&spec.theme.font_family),
                    esc_text(label)
                ),
            );
        }
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
        if !node.node.metrics.is_empty() {
            render_metric_node(&mut svg, node, spec, &color);
            continue;
        }
        push_line(
            &mut svg,
            &format!(
                r##"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="{:.1}" fill="#ffffff" stroke="{color}" stroke-width="{:.1}"/>"##,
                node.x,
                node.y,
                node.w,
                node.h,
                (node.h * 0.14).clamp(2.5, 7.0),
                if node.h < 24.0 { 0.9 } else { 1.4 }
            ),
        );
        let stripe_h = (node.h - 10.0).max(6.0);
        push_line(
            &mut svg,
            &format!(
                r##"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{stripe_h:.1}" rx="{:.1}" fill="{color}"/>"##,
                node.x + 8.0,
                node.y + (node.h - stripe_h) / 2.0,
                if node.h < 24.0 { 3.0 } else { 5.0 },
                if node.h < 24.0 { 1.5 } else { 2.5 }
            ),
        );
        let label_font = (node.h * 0.25).clamp(7.0, 13.0);
        let detail_font = (node.h * 0.20).clamp(6.0, 10.5);
        let label_lines = if node.h < 26.0 { 1 } else { 2 };
        push_wrapped_text(
            &mut svg,
            &node.node.label,
            node.x + 22.0,
            node.y + (node.h * 0.45).max(label_font + 2.0),
            node.w - 32.0,
            label_font,
            label_lines,
            &spec.theme.ink,
            &spec.theme.font_family,
            true,
        );
        if node.h >= 34.0
            && let Some(detail) = &node.node.detail
        {
            push_wrapped_text(
                &mut svg,
                detail,
                node.x + 22.0,
                node.y + node.h - 11.0,
                node.w - 32.0,
                detail_font,
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

fn should_show_lane_headers(
    groups: &[&crate::schema::GraphGroup],
    by_group: &HashMap<&str, Vec<&GraphNode>>,
) -> bool {
    !groups.iter().all(|group| {
        by_group
            .get(group.id.as_str())
            .is_some_and(|nodes| nodes.len() == 1 && !nodes[0].metrics.is_empty())
    })
}

fn render_metric_node(svg: &mut String, node: &PlacedNode<'_>, spec: &GraphSpec, color: &str) {
    push_line(
        svg,
        &format!(
            r##"<rect class="metric-card" x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="7" fill="#ffffff" stroke="{color}" stroke-width="1.5"/>"##,
            node.x, node.y, node.w, node.h
        ),
    );
    push_line(
        svg,
        &format!(
            r##"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="4.5" rx="2.2" fill="{color}"/>"##,
            node.x + 12.0,
            node.y + 11.0,
            node.w - 24.0,
        ),
    );
    push_wrapped_text(
        svg,
        &node.node.label,
        node.x + 14.0,
        node.y + 36.0,
        node.w - 28.0,
        14.0,
        1,
        &spec.theme.ink,
        &spec.theme.font_family,
        true,
    );
    if let Some(detail) = &node.node.detail {
        push_wrapped_text(
            svg,
            detail,
            node.x + 14.0,
            node.y + 53.0,
            node.w - 28.0,
            8.6,
            2,
            &spec.theme.muted,
            &spec.theme.font_family,
            false,
        );
    }

    let metric_top = node.y + 78.0;
    let row_h = 24.0;
    for (idx, metric) in node.node.metrics.iter().enumerate() {
        let y = metric_top + idx as f32 * row_h;
        if y + row_h > node.y + node.h - 9.0 {
            break;
        }
        push_line(
            svg,
            &format!(
                r##"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="4" fill="{color}" opacity="0.075"/>"##,
                node.x + 12.0,
                y,
                node.w - 24.0,
                row_h - 5.0,
            ),
        );
        push_wrapped_text(
            svg,
            &metric.label,
            node.x + 20.0,
            y + 12.6,
            (node.w - 40.0) * 0.42,
            7.8,
            1,
            &spec.theme.muted,
            &spec.theme.font_family,
            true,
        );
        push_wrapped_text(
            svg,
            &metric.value,
            node.x + 20.0 + (node.w - 40.0) * 0.43,
            y + 12.6,
            (node.w - 40.0) * 0.57,
            8.3,
            1,
            &spec.theme.ink,
            &spec.theme.font_family,
            false,
        );
    }
}

fn render_overview_detail_edges(
    svg: &mut String,
    edges: &[&GraphEdge],
    placed: &HashMap<&str, PlacedNode<'_>>,
    spec: &GraphSpec,
) {
    let mut by_source = BTreeMap::<&str, Vec<&GraphEdge>>::new();
    for edge in edges
        .iter()
        .copied()
        .filter(|edge| edge.kind == "overview-detail")
    {
        by_source
            .entry(edge.source.as_str())
            .or_default()
            .push(edge);
    }

    for (source_id, mut source_edges) in by_source {
        source_edges.sort_by(|a, b| {
            let ay = placed
                .get(a.target.as_str())
                .map(|node| node.y)
                .unwrap_or_default();
            let by = placed
                .get(b.target.as_str())
                .map(|node| node.y)
                .unwrap_or_default();
            ay.partial_cmp(&by)
                .unwrap()
                .then_with(|| a.target.cmp(&b.target))
        });
        let Some(source) = placed.get(source_id) else {
            continue;
        };
        let targets = source_edges
            .iter()
            .filter_map(|edge| placed.get(edge.target.as_str()))
            .collect::<Vec<_>>();
        let Some(last_target) = targets.last() else {
            continue;
        };

        let color = kind_color(&source.node.kind, &spec.theme.accent);
        let source_x = source.x + source.w / 2.0;
        let source_y = source.y + source.h;
        let spine_x = source.x - 10.0;
        let spine_top = source_y + 9.0;
        let spine_bottom = last_target.y + last_target.h / 2.0;
        push_line(
            svg,
            &format!(
                r##"<path class="overview-detail-spine" d="M {source_x:.1} {source_y:.1} L {source_x:.1} {spine_top:.1} L {spine_x:.1} {spine_top:.1} L {spine_x:.1} {spine_bottom:.1}" fill="none" stroke="{color}" stroke-width="1.2" opacity="0.42" stroke-linecap="round" stroke-linejoin="round"/>"##
            ),
        );
        for target in targets {
            let branch_y = target.y + target.h / 2.0;
            let branch_x = target.x + 8.0;
            push_line(
                svg,
                &format!(
                    r##"<path class="overview-detail-branch" d="M {spine_x:.1} {branch_y:.1} L {branch_x:.1} {branch_y:.1}" fill="none" stroke="{color}" stroke-width="1.05" opacity="0.44" stroke-linecap="round"/>"##
                ),
            );
        }
    }
}

pub fn render_document_svg(
    document: &TopoxideDocument,
    profile_id: Option<&str>,
    options: RenderOptions,
) -> Result<String, ValidationError> {
    let view = apply_profile(document, profile_id)?;
    render_profile_view_svg(&view, options)
}

pub fn render_profile_view_svg(
    view: &ProfileView,
    options: RenderOptions,
) -> Result<String, ValidationError> {
    if view.profile.view == ProfileViewKind::Matrix
        && let Some(matrix) = &view.matrix
    {
        return Ok(render_matrix_svg(
            &view.graph,
            &view.profile.label,
            matrix,
            options,
        ));
    }
    render_lane_graph_svg(&view.graph, options, view.profile.show_lane_enclosures)
}

fn render_matrix_svg(
    spec: &GraphSpec,
    profile_label: &str,
    matrix: &MatrixView,
    options: RenderOptions,
) -> String {
    let margin = 56.0;
    let title_h = 96.0;
    let row_label_w = 280.0;
    let top = margin + title_h;
    let table_w = options.width - margin * 2.0;
    let table_h = options.height - top - margin - 32.0;
    let col_count = matrix.columns.len().max(1);
    let row_count = matrix.rows.len().max(1);
    let col_w = (table_w - row_label_w) / col_count as f32;
    let row_h = (table_h / (row_count as f32 + 1.0)).clamp(18.0, 42.0);
    let max_count = matrix
        .rows
        .iter()
        .flat_map(|row| row.cells.iter().map(|cell| cell.count))
        .max()
        .unwrap_or(0);

    let mut svg = String::new();
    push_line(
        &mut svg,
        &format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{:.0}" height="{:.0}" viewBox="0 0 {:.0} {:.0}" role="img" aria-label="{} {}">"##,
            options.width,
            options.height,
            options.width,
            options.height,
            esc_attr(&spec.title),
            esc_attr(profile_label),
        ),
    );
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
            r##"<text x="{margin}" y="48" fill="{}" font-family="{}" font-size="26" font-weight="600">{}</text>"##,
            spec.theme.ink,
            esc_attr(&spec.theme.font_family),
            esc_text(profile_label)
        ),
    );
    push_line(
        &mut svg,
        &format!(
            r##"<text x="{margin}" y="76" fill="{}" font-family="{}" font-size="13">{}</text>"##,
            spec.theme.muted,
            esc_attr(&spec.theme.font_family),
            esc_text("Rows are source datasets; cells count matching ETL coverage links.")
        ),
    );

    push_line(
        &mut svg,
        &format!(
            r##"<rect x="{margin:.1}" y="{top:.1}" width="{table_w:.1}" height="{:.1}" rx="8" fill="#ffffff" stroke="{}" stroke-width="1"/>"##,
            row_h * (row_count as f32 + 1.0),
            spec.theme.grid
        ),
    );
    for (idx, column) in matrix.columns.iter().enumerate() {
        let x = margin + row_label_w + idx as f32 * col_w;
        push_wrapped_text(
            &mut svg,
            &column.label,
            x + 8.0,
            top + 24.0,
            col_w - 16.0,
            10.0,
            2,
            &spec.theme.ink,
            &spec.theme.font_family,
            true,
        );
    }
    for (row_idx, row) in matrix.rows.iter().enumerate() {
        let y = top + row_h * (row_idx as f32 + 1.0);
        let fill = if row_idx % 2 == 0 {
            "#fbfbf8"
        } else {
            "#ffffff"
        };
        push_line(
            &mut svg,
            &format!(
                r##"<rect x="{margin:.1}" y="{y:.1}" width="{table_w:.1}" height="{row_h:.1}" fill="{fill}"/>"##
            ),
        );
        push_wrapped_text(
            &mut svg,
            &row.node.label,
            margin + 12.0,
            y + row_h * 0.62,
            row_label_w - 24.0,
            10.0,
            1,
            &spec.theme.ink,
            &spec.theme.font_family,
            true,
        );
        for (col_idx, cell) in row.cells.iter().enumerate() {
            let x = margin + row_label_w + col_idx as f32 * col_w;
            let active = cell.count > 0;
            let (color, text_color) = matrix_cell_colors(cell.count, max_count, &spec.theme.grid);
            let label = if active {
                cell.count.to_string()
            } else {
                String::new()
            };
            push_line(
                &mut svg,
                &format!(
                    r##"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="4" fill="{}" opacity="{}"/>"##,
                    x + 8.0,
                    y + 4.0,
                    col_w - 16.0,
                    row_h - 8.0,
                    color,
                    if active { "0.92" } else { "0.18" },
                ),
            );
            if active {
                push_line(
                    &mut svg,
                    &format!(
                        r##"<text x="{:.1}" y="{:.1}" fill="{text_color}" font-family="{}" font-size="10" text-anchor="middle" font-weight="700">{}</text>"##,
                        x + col_w / 2.0,
                        y + row_h * 0.62,
                        esc_attr(&spec.theme.font_family),
                        esc_text(&label)
                    ),
                );
            }
        }
    }
    push_line(&mut svg, "</svg>");
    svg
}

fn adaptive_node_height(available_h: f32, count: usize) -> f32 {
    if count <= 1 {
        return 56.0;
    }
    let count = count as f32;
    let target_gap = if count > 36.0 {
        3.0
    } else if count > 24.0 {
        5.0
    } else {
        10.0
    };
    ((available_h - target_gap * (count - 1.0)) / count).clamp(13.0, 56.0)
}

fn matrix_cell_colors(count: usize, max_count: usize, empty_color: &str) -> (String, &'static str) {
    if count == 0 || max_count == 0 {
        return (empty_color.to_owned(), "#1f2933");
    }
    let ratio = count as f32 / max_count as f32;
    let fill = if ratio <= 0.25 {
        "#d8ecf8"
    } else if ratio <= 0.50 {
        "#9bd2ee"
    } else if ratio <= 0.75 {
        "#56B4E9"
    } else {
        "#0072B2"
    };
    let text = if ratio <= 0.50 { "#1f2933" } else { "#ffffff" };
    (fill.to_owned(), text)
}

fn adaptive_node_height_for_nodes(available_h: f32, nodes: &[&GraphNode]) -> f32 {
    let count = nodes.len().max(1);
    let metric_count = nodes
        .iter()
        .map(|node| node.metrics.len())
        .max()
        .unwrap_or_default();
    if metric_count == 0 {
        return adaptive_node_height(available_h, count);
    }
    let desired = (96.0 + metric_count as f32 * 24.0).clamp(124.0, 190.0);
    if count <= 1 {
        return desired.min(available_h.max(72.0));
    }
    let normal = adaptive_node_height(available_h, count);
    desired.min(normal.max(72.0))
}

fn adaptive_step(available_h: f32, count: usize, node_h: f32) -> f32 {
    if count <= 1 {
        return 0.0;
    }
    ((available_h - node_h) / (count - 1) as f32).max(node_h + 1.0)
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
    if kinds.len() <= 1 {
        return;
    }
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
    use crate::profile::{MatrixCell, MatrixRow, MatrixView, apply_profile};
    use crate::schema::{
        EdgeMode, GraphEdge, GraphGroup, GraphNode, GraphNodeMetric, GraphProfile, GraphSpec,
        GraphTheme, ProfileViewKind, TopoxideDocument,
    };

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
                    metrics: Vec::new(),
                    order: 0,
                    weight: None,
                },
                GraphNode {
                    id: "n1".into(),
                    label: "Source".into(),
                    kind: "dataset".into(),
                    group: "a".into(),
                    detail: None,
                    metrics: Vec::new(),
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
        assert!(a.contains("class=\"lane-enclosure\""));
        assert!(a.contains(">dataset</text>"));
        assert!(a.contains(">clickhouse-table</text>"));
    }

    #[test]
    fn overview_detail_edges_render_as_marker_free_bracket_spines() {
        let spec = GraphSpec {
            title: "Overview".into(),
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
                    id: "summary".into(),
                    label: "Summary".into(),
                    kind: "overview".into(),
                    group: "a".into(),
                    detail: None,
                    metrics: Vec::new(),
                    order: 0,
                    weight: None,
                },
                GraphNode {
                    id: "detail-a".into(),
                    label: "Detail A".into(),
                    kind: "overview-detail".into(),
                    group: "a".into(),
                    detail: None,
                    metrics: Vec::new(),
                    order: 1,
                    weight: None,
                },
                GraphNode {
                    id: "detail-b".into(),
                    label: "Detail B".into(),
                    kind: "overview-detail".into(),
                    group: "a".into(),
                    detail: None,
                    metrics: Vec::new(),
                    order: 2,
                    weight: None,
                },
                GraphNode {
                    id: "detail-c".into(),
                    label: "Detail C".into(),
                    kind: "overview-detail".into(),
                    group: "a".into(),
                    detail: None,
                    metrics: Vec::new(),
                    order: 3,
                    weight: None,
                },
                GraphNode {
                    id: "target".into(),
                    label: "Target".into(),
                    kind: "consumer".into(),
                    group: "b".into(),
                    detail: None,
                    metrics: Vec::new(),
                    order: 0,
                    weight: None,
                },
            ],
            edges: vec![
                GraphEdge {
                    source: "summary".into(),
                    target: "detail-a".into(),
                    kind: "overview-detail".into(),
                    label: None,
                },
                GraphEdge {
                    source: "summary".into(),
                    target: "detail-b".into(),
                    kind: "overview-detail".into(),
                    label: None,
                },
                GraphEdge {
                    source: "summary".into(),
                    target: "detail-c".into(),
                    kind: "overview-detail".into(),
                    label: None,
                },
                GraphEdge {
                    source: "summary".into(),
                    target: "target".into(),
                    kind: "overview-flow".into(),
                    label: None,
                },
            ],
            theme: GraphTheme::default(),
        };

        let svg = render_svg(&spec, RenderOptions::default()).unwrap();
        assert!(svg.contains("class=\"overview-detail-spine\""));
        assert_eq!(svg.matches("class=\"overview-detail-spine\"").count(), 1);
        assert_eq!(svg.matches("class=\"overview-detail-branch\"").count(), 3);
        assert!(svg.contains("marker-end=\"url(#arrow)\""));
        for line in svg.lines().filter(|line| line.contains("overview-detail-")) {
            assert!(!line.contains("marker-end"));
        }
    }

    #[test]
    fn metric_nodes_render_as_single_cards() {
        let spec = GraphSpec {
            title: "Metrics".into(),
            subtitle: None,
            groups: vec![GraphGroup {
                id: "a".into(),
                label: "A".into(),
                order: 0,
                description: None,
            }],
            nodes: vec![GraphNode {
                id: "summary".into(),
                label: "Summary".into(),
                kind: "overview".into(),
                group: "a".into(),
                detail: Some("Production summary".into()),
                metrics: vec![
                    GraphNodeMetric {
                        label: "Families".into(),
                        value: "7 source types".into(),
                    },
                    GraphNodeMetric {
                        label: "Scale".into(),
                        value: "small to massive".into(),
                    },
                    GraphNodeMetric {
                        label: "Variants".into(),
                        value: "12 dataset variants".into(),
                    },
                ],
                order: 0,
                weight: None,
            }],
            edges: Vec::new(),
            theme: GraphTheme::default(),
        };

        let svg = render_svg(&spec, RenderOptions::default()).unwrap();
        assert!(svg.contains("class=\"lane-enclosure\""));
        assert!(svg.contains("class=\"metric-card\""));
        assert!(svg.contains("stroke="));
        assert!(!svg.contains("class=\"lane-header\""));
        assert!(!svg.contains(">overview</text>"));
        assert!(svg.contains("Families"));
        assert!(svg.contains("Variants"));
    }

    #[test]
    fn profile_can_hide_lane_enclosures_without_hiding_edges() {
        let spec = GraphSpec {
            title: "Metrics".into(),
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
                    id: "source".into(),
                    label: "Source".into(),
                    kind: "overview".into(),
                    group: "a".into(),
                    detail: Some("Production summary".into()),
                    metrics: vec![GraphNodeMetric {
                        label: "Families".into(),
                        value: "7 source types".into(),
                    }],
                    order: 0,
                    weight: None,
                },
                GraphNode {
                    id: "target".into(),
                    label: "Target".into(),
                    kind: "overview".into(),
                    group: "b".into(),
                    detail: Some("Serving summary".into()),
                    metrics: vec![GraphNodeMetric {
                        label: "Consumers".into(),
                        value: "4 uses".into(),
                    }],
                    order: 0,
                    weight: None,
                },
            ],
            edges: vec![GraphEdge {
                source: "source".into(),
                target: "target".into(),
                kind: "overview-flow".into(),
                label: None,
            }],
            theme: GraphTheme::default(),
        };
        let document = TopoxideDocument {
            graph: spec,
            profiles: vec![GraphProfile {
                id: "overview".into(),
                label: "Overview".into(),
                description: None,
                view: ProfileViewKind::LaneGraph,
                edge_mode: EdgeMode::All,
                groups: Vec::new(),
                node_kinds: Vec::new(),
                edge_kinds: Vec::new(),
                show_lane_enclosures: false,
                matrix: None,
            }],
        };
        let view = apply_profile(&document, Some("overview")).unwrap();

        let svg = render_profile_view_svg(&view, RenderOptions::default()).unwrap();
        assert!(!svg.contains("class=\"lane-enclosure\""));
        assert!(svg.contains("class=\"metric-card\""));
        assert!(svg.contains("marker-end=\"url(#arrow)\""));
        assert!(svg.contains("stroke="));
    }

    #[test]
    fn matrix_cells_are_colored_by_count() {
        let spec = GraphSpec {
            title: "Matrix".into(),
            subtitle: None,
            groups: Vec::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
            theme: GraphTheme::default(),
        };
        let matrix = MatrixView {
            columns: vec![
                crate::schema::MatrixColumn {
                    id: "one".into(),
                    label: "One".into(),
                    edge_kinds: Vec::new(),
                    target_groups: Vec::new(),
                    target_kinds: Vec::new(),
                },
                crate::schema::MatrixColumn {
                    id: "two".into(),
                    label: "Two".into(),
                    edge_kinds: Vec::new(),
                    target_groups: Vec::new(),
                    target_kinds: Vec::new(),
                },
                crate::schema::MatrixColumn {
                    id: "three".into(),
                    label: "Three".into(),
                    edge_kinds: Vec::new(),
                    target_groups: Vec::new(),
                    target_kinds: Vec::new(),
                },
            ],
            rows: vec![MatrixRow {
                node: GraphNode {
                    id: "row".into(),
                    label: "Row".into(),
                    kind: "dataset".into(),
                    group: "sources".into(),
                    detail: None,
                    metrics: Vec::new(),
                    order: 0,
                    weight: None,
                },
                cells: vec![
                    MatrixCell { count: 1 },
                    MatrixCell { count: 2 },
                    MatrixCell { count: 3 },
                ],
            }],
        };

        let svg = render_matrix_svg(&spec, "Coverage matrix", &matrix, RenderOptions::default());
        assert!(svg.contains(r##"fill="#9bd2ee""##));
        assert!(svg.contains(r##"fill="#56B4E9""##));
        assert!(svg.contains(r##"fill="#0072B2""##));
        assert!(svg.contains(r##"fill="#ffffff""##));
    }
}
