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
    pub order: usize,
    #[serde(default)]
    pub weight: Option<f32>,
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
