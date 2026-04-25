# Manuscript Styling

The default theme is tuned for scientific figures:

- Warm off-white background.
- High-contrast ink.
- Muted grid and lane labels.
- A restrained blue accent.
- SVG text using a manuscript-friendly sans-serif stack.

Override theme fields in the spec when a project needs its own palette:

```toml
[theme]
background = "#fbfbf8"
ink = "#1f2933"
muted = "#667085"
grid = "#d9d6c9"
accent = "#0072b2"
font-family = "Latin Modern Sans, Inter, Helvetica, Arial, sans-serif"
```

For publication workflows:

- Render SVG with fixed dimensions.
- Keep generated SVG out of source control unless it is a deliberate fixture.
- Prefer short node labels and use `detail` for secondary information.
- Validate generated specs in CI before rendering composite figures.
