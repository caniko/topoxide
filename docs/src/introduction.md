# topoxide

`topoxide` renders deterministic, layered graph figures from a typed graph
specification. It is built for scientific and engineering diagrams where the
source of truth should be data, not a hand-edited drawing.

The renderer is domain-neutral. A project generates a `GraphSpec` from its own
registry, schema, workflow, or metadata, and `topoxide` validates, lays out, and
renders the graph as SVG.

Core properties:

- Stable ordering for reproducible figures.
- Typed nodes, edges, groups, and theme fields.
- JSON and TOML input for generated or hand-authored specs.
- SVG output suitable for manuscripts, documentation, and static sites.
- A small Rust API for embedding render steps in existing pipelines.

The first production consumer is SynDB, where a generated graph shows how ETL
jobs, object storage artifacts, ClickHouse tables, materialized views,
precomputed graph products, and downstream consumers connect.

Source: [codeberg.org/caniko/topoxide](https://codeberg.org/caniko/topoxide)
