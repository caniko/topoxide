# Rust API

The public API is intentionally small:

```rust
use topoxide::{GraphSpec, RenderOptions, render_svg, validate_graph};

fn render(spec: &GraphSpec) -> Result<String, Box<dyn std::error::Error>> {
    validate_graph(spec)?;
    let svg = render_svg(
        spec,
        RenderOptions {
            width: 1800.0,
            height: 1200.0,
        },
    )?;
    Ok(svg)
}
```

Important types:

- `GraphSpec`
- `GraphGroup`
- `GraphNode`
- `GraphEdge`
- `GraphTheme`
- `RenderOptions`

Use the Rust API when the graph spec is generated inside an existing Rust
pipeline. Use the CLI when the spec is emitted as JSON or TOML by another tool.
