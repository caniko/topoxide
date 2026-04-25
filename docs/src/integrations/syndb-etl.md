# SynDB ETL Graph

SynDB generates a `topoxide` graph from production metadata and renders it as a
supplementary manuscript figure.

The SynDB adapter reads:

- ETL runtime registry.
- Shared dataset identity metadata.
- Helm ETL job structure.
- ClickHouse table hierarchy.
- ClickHouse materialized views.
- Precomputed graph tables.
- Downstream API and manuscript consumers.

The generated graph includes lanes for:

- Sources.
- ETL runtime.
- Object storage.
- SynDB tables.
- Derived views.
- Graph products.
- Consumers.

In SynDB, regenerate the figure with:

```bash
just cf 12
```

The generated panel spec and SVG are build artifacts. The source of truth is the
production metadata plus the adapter code in SynDB.
