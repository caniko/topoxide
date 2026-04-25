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

## Website and documentation

The project includes a Codeberg Pages site and mdBook documentation:

```bash
nix build .#website
nix build .#docs
nix build .#site
```

For local development:

```bash
cd website && zola serve
cd docs && mdbook serve
```
