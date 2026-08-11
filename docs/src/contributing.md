# Contributing

Thanks for helping build Parallax.

Canonical short guide (setup commands, DCO note, PR norms):  
**[CONTRIBUTING.md](https://github.com/parallax-runtime/parallax/blob/main/CONTRIBUTING.md)** in the repository root.

Also see the [Code of Conduct](https://github.com/parallax-runtime/parallax/blob/main/CODE_OF_CONDUCT.md), [Security policy](https://github.com/parallax-runtime/parallax/blob/main/SECURITY.md), and [Privacy policy](https://github.com/parallax-runtime/parallax/blob/main/PRIVACY.md).

## Development setup

```bash
git clone https://github.com/parallax-runtime/parallax.git
cd parallax
cargo build --workspace
cargo test --workspace
cargo run -p parallax-cli --bin plx -- doctor
```

Prerequisites: Rust 1.75+, Node.js 18+ (JS adapter), Python 3.10+ (Python adapter).

### Docs site

```bash
cd docs
mdbook serve --open
# http://localhost:3000
```

Build static site:

```bash
mdbook build
# output: docs/book/
```

Requires [mdBook](https://rust-lang.github.io/mdBook/) (`cargo install mdbook`).

## Project norms

- **No fake capabilities** — return `Unsupported` / structured errors
- Prefer small, focused PRs
- Use conventional commits: `feat:`, `fix:`, `docs:`, `chore:`, `test:`, `ci:`
- Keep README and docs synchronized with real CLI behavior
- Add tests for migration / PIR / snapshot critical paths
- Update [CHANGELOG.md](https://github.com/parallax-runtime/parallax/blob/main/CHANGELOG.md) for user-visible changes
- Bump the correct [versioning](./versioning.md) surface when you break PIR, protocol, snapshots, or adapter contracts

## Adapter conformance

See root [CONTRIBUTING.md](https://github.com/parallax-runtime/parallax/blob/main/CONTRIBUTING.md#adapter-conformance) and [Adapters overview](./adapters/overview.md). In short: honest capabilities, versioned protocol, structured errors, lenient probe/registration, tests for paths you touch.

## Code layout

See [Architecture](./architecture.md). Adapters live under `crates/parallax-adapter-*` with worker scripts in `adapters/`.

## Checks before opening a PR

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd docs && mdbook build
```

CI runs these on Linux, Windows, and macOS (plus docs + audit jobs).

## License

Contributions are accepted under the Apache-2.0 license. No CLA is required at this time; see the root contributing guide for the lightweight DCO note.
