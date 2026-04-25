# topoxide

`topoxide` renders deterministic, layered graph figures from a typed JSON or
TOML graph specification. It is intentionally domain-neutral: projects generate
the graph spec from their production metadata, and `topoxide` handles validation,
layout, and SVG export.

The first downstream use is the SynDB ETL-to-SynDB manuscript figure.

## CLI

```bash
topoxide validate --input graph.toml
topoxide render --input graph.toml --output figure.svg
topoxide schema --output topoxide.schema.json
```

JSON and TOML inputs use the same schema.
