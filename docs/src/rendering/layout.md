# Layout Model

`topoxide` uses a deterministic lane layout. Each group becomes a horizontal
band. Nodes are placed inside their group using stable ordering, and edges are
routed between node centers.

This model is intentionally conservative:

- It avoids force-directed randomness.
- It favors repeatable manuscript output over exploratory graph layout.
- It keeps lane semantics visible even in large generated graphs.
- It allows adapter code to provide ordering hints without hand-maintaining the
  full node list.

Use `GraphNode.order` for local ordering within a group. Use `GraphGroup.order`
for lane ordering across the figure.

The current renderer targets dense data-flow and data-model diagrams. It is not
intended to replace interactive graph exploration tools.
