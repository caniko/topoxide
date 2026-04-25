# Production Metadata Adapters

`topoxide` does not read deployment systems, databases, or ETL registries
directly. The owning project should provide an adapter that turns production
metadata into a `GraphSpec`.

Typical adapter inputs:

- Dataset registries.
- Runtime job definitions.
- Object storage prefixes or artifacts.
- Database table names and column metadata.
- View definitions and dependency maps.
- Batch products and downstream consumers.

The adapter should be deterministic. Use stable maps, stable sorting, and
deduplicated edges so the same metadata produces the same SVG.

Recommended checks:

- Every generated edge endpoint resolves to a node.
- Every import table maps to a known table enum or schema entry.
- Every view dependency resolves to a graph node.
- The emitted spec validates with `topoxide validate`.
