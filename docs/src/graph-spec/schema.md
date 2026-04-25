# Schema

The CLI can write a JSON Schema document:

```bash
topoxide schema --output topoxide.schema.json
```

The core fields are:

```text
GraphSpec
  title: string
  subtitle: optional string
  groups: GraphGroup[]
  nodes: GraphNode[]
  edges: GraphEdge[]
  theme: optional GraphTheme
```

`GraphGroup`:

```text
id: string
label: string
order: integer
description: optional string
```

`GraphNode`:

```text
id: string
label: string
kind: string
group: string
detail: optional string
order: integer
weight: optional number
```

`GraphEdge`:

```text
source: string
target: string
kind: string
label: optional string
```

`GraphTheme`:

```text
background: color
ink: color
muted: color
grid: color
accent: color
font-family: CSS font stack
```

Validation checks that group ids are unique, node ids are unique, node groups
exist, and edge endpoints resolve to known nodes.
