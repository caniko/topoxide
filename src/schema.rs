use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GraphSpec {
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub groups: Vec<GraphGroup>,
    pub nodes: Vec<GraphNode>,
    #[serde(default)]
    pub edges: Vec<GraphEdge>,
    #[serde(default)]
    pub theme: GraphTheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TopoxideDocument {
    pub graph: GraphSpec,
    #[serde(default)]
    pub profiles: Vec<GraphProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GraphProfile {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub view: ProfileViewKind,
    #[serde(default)]
    pub edge_mode: EdgeMode,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub node_kinds: Vec<String>,
    #[serde(default)]
    pub edge_kinds: Vec<String>,
    #[serde(default = "default_show_lane_enclosures")]
    pub show_lane_enclosures: bool,
    #[serde(default)]
    pub matrix: Option<MatrixProfile>,
}

pub fn default_show_lane_enclosures() -> bool {
    true
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ProfileViewKind {
    #[default]
    LaneGraph,
    Matrix,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EdgeMode {
    #[default]
    All,
    Bundled,
    Hidden,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MatrixProfile {
    pub row_group: String,
    #[serde(default)]
    pub columns: Vec<MatrixColumn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MatrixColumn {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub edge_kinds: Vec<String>,
    #[serde(default)]
    pub target_groups: Vec<String>,
    #[serde(default)]
    pub target_kinds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GraphGroup {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub order: usize,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub group: String,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(default)]
    pub metrics: Vec<GraphNodeMetric>,
    #[serde(default)]
    pub order: usize,
    #[serde(default)]
    pub weight: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GraphNodeMetric {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GraphTheme {
    #[serde(default = "GraphTheme::default_background")]
    pub background: String,
    #[serde(default = "GraphTheme::default_ink")]
    pub ink: String,
    #[serde(default = "GraphTheme::default_muted")]
    pub muted: String,
    #[serde(default = "GraphTheme::default_grid")]
    pub grid: String,
    #[serde(default = "GraphTheme::default_accent")]
    pub accent: String,
    #[serde(default = "GraphTheme::default_font_family")]
    pub font_family: String,
}

impl Default for GraphTheme {
    fn default() -> Self {
        Self {
            background: Self::default_background(),
            ink: Self::default_ink(),
            muted: Self::default_muted(),
            grid: Self::default_grid(),
            accent: Self::default_accent(),
            font_family: Self::default_font_family(),
        }
    }
}

impl GraphTheme {
    fn default_background() -> String {
        "#fbfbf8".to_owned()
    }

    fn default_ink() -> String {
        "#1f2933".to_owned()
    }

    fn default_muted() -> String {
        "#667085".to_owned()
    }

    fn default_grid() -> String {
        "#d9d6c9".to_owned()
    }

    fn default_accent() -> String {
        "#0072b2".to_owned()
    }

    fn default_font_family() -> String {
        "Latin Modern Sans, Inter, Helvetica, Arial, sans-serif".to_owned()
    }
}
