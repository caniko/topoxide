+++
title = "topoxide"

[extra]
tagline = "Data-driven graph figures for scientific systems."
subtitle = "Render typed ETL, schema, and workflow graphs into deterministic SVG without turning production architecture into a hand-edited illustration."
install = "cargo install --git https://codeberg.org/caniko/topoxide.git"

[[extra.features]]
title = "Typed graph spec"
body = "Nodes, edges, groups, and themes are serialized as JSON or TOML, so adapters can emit figures from real metadata."

[[extra.features]]
title = "Deterministic layout"
body = "Stable lane and node ordering keeps generated SVG output reproducible across CI, manuscript builds, and releases."

[[extra.features]]
title = "Manuscript-ready SVG"
body = "The renderer targets clean, high-contrast, publication-oriented figures rather than interactive exploration."

[[extra.features]]
title = "Generic by design"
body = "topoxide does not know about SynDB, Helm, ClickHouse, or S3. Projects own adapters; topoxide owns rendering."

[[extra.features]]
title = "Small Rust API"
body = "Embed validation and SVG rendering directly in Rust pipelines, or use the CLI from any language that can write a spec."

[[extra.features]]
title = "Docs and build outputs"
body = "A Nix flake builds the renderer, presentation site, mdBook docs, and a combined Codeberg Pages artifact."
+++
