# Parallax documentation (mdBook)

<p align="center">
  <img src="assets/parallax-logo.png" alt="Parallax" width="96">
</p>

This directory is the source for the GitHub Pages site:

**https://parallax-runtime.github.io/parallax/**

## Preview locally

```bash
# install once
cargo install mdbook

cd docs
mdbook serve --open
```

Opens a live-reloading site (default `http://localhost:3000`) with sidebar navigation and search.

## Build

```bash
cd docs
mdbook build
# static output → docs/book/
```

CI builds the book on every PR (`ci.yml`). Deploys to Pages on pushes to `main` that touch `docs/**` (`pages.yml`).

Project governance at the repository root (linked from the book’s Project section):

- [CHANGELOG.md](../CHANGELOG.md)
- [CONTRIBUTING.md](../CONTRIBUTING.md)
- [SECURITY.md](../SECURITY.md)
- [PRIVACY.md](../PRIVACY.md)
- [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md)
