# Quick Start

Create a graph spec:

```toml
title = "ETL flow"
subtitle = "Minimal topoxide graph"

[[groups]]
id = "sources"
label = "Sources"
order = 0

[[groups]]
id = "tables"
label = "Tables"
order = 1

[[nodes]]
id = "source.ciona"
label = "Ciona source"
kind = "dataset"
group = "sources"
order = 0

[[nodes]]
id = "table.synapses"
label = "Synapses"
kind = "clickhouse-table"
group = "tables"
order = 0

[[edges]]
source = "source.ciona"
target = "table.synapses"
kind = "imports"
label = "ETL"
```

Validate and render it:

```bash
topoxide validate --input graph.toml
topoxide render --input graph.toml --output figure.svg
```

Use explicit dimensions when preparing a figure for a manuscript or poster:

```bash
topoxide render \
  --input graph.toml \
  --output figure.svg \
  --width 1800 \
  --height 1200
```
