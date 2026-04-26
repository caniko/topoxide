pub mod input;
#[cfg(feature = "leptos")]
pub mod leptos_view;
pub mod profile;
pub mod render;
pub mod schema;
pub mod validate;

pub use input::{InputError, TopoxideInput, read_input};
pub use profile::{MatrixCell, MatrixRow, MatrixView, ProfileView, apply_profile};
pub use render::{RenderOptions, render_document_svg, render_profile_view_svg, render_svg};
pub use schema::{
    EdgeMode, GraphEdge, GraphGroup, GraphNode, GraphNodeMetric, GraphProfile, GraphSpec,
    GraphTheme, MatrixColumn, MatrixProfile, ProfileViewKind, TopoxideDocument,
};
pub use validate::{ValidationError, validate_graph};
