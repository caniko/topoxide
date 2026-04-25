# Overview

A `GraphSpec` describes the figure that `topoxide` renders. It has four main
parts:

- Groups define horizontal lanes.
- Nodes define typed entities placed inside groups.
- Edges define directed relationships between nodes.
- Theme fields define colors and typography.

The renderer uses stable sorting. Groups are ordered by `order`, then by `id`.
Nodes are ordered by group, `order`, then `id`. This keeps generated SVG output
deterministic across runs.

Recommended adapter pattern:

1. Read production metadata from the owning project.
2. Convert that metadata into a `GraphSpec`.
3. Validate the spec with `topoxide validate`.
4. Render SVG with fixed width and height.
5. Treat generated SVGs as build artifacts.

This keeps the visual diagram synchronized with the system it describes.
