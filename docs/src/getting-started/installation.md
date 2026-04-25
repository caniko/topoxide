# Installation

Install from the Codeberg repository:

```bash
cargo install --git https://codeberg.org/caniko/topoxide.git
```

For local development:

```bash
git clone https://codeberg.org/caniko/topoxide.git
cd topoxide
cargo test
```

If you use Nix:

```bash
nix develop
cargo test
```

The flake also exposes the renderer and site outputs:

```bash
nix build .#topoxide
nix build .#website
nix build .#docs
nix build .#site
```
