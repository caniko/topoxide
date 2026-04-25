# CLI

`topoxide` exposes three commands.

Render:

```bash
topoxide render --input graph.toml --output figure.svg
```

Options:

```text
--input <PATH>    JSON or TOML graph spec
--output <PATH>   SVG output path
--width <FLOAT>   output width, default 1800
--height <FLOAT>  output height, default 1200
```

Validate:

```bash
topoxide validate --input graph.toml
```

Schema:

```bash
topoxide schema --output topoxide.schema.json
```

Supported input extensions are `.json` and `.toml`.
