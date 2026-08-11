<p align="center">
  <img src="docs/assets/parallax-logo.png" alt="Parallax logo" width="128">
</p>

<h1 align="center">Parallax</h1>

<p align="center">
  <strong>Polyglot migration and universal execution runtime</strong><br>
  Capture program state in one language, encode it as language-neutral IR, restore or migrate it in another.
</p>

<p align="center">
  <a href="https://github.com/theworker02/Parallax/actions/workflows/ci.yml"><img src="https://github.com/theworker02/Parallax/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache%202.0-blue.svg" alt="License"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.75%2B-orange.svg?logo=rust" alt="Rust"></a>
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/version-0.1.0-informational" alt="Version"></a>
  <a href="https://parallax-runtime.github.io/parallax/"><img src="https://img.shields.io/badge/docs-GitHub%20Pages-0B5FFF?logo=gitbook&logoColor=white" alt="Docs"></a>
  <a href="https://github.com/theworker02/Parallax/actions/workflows/ci.yml"><img src="https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-informational" alt="Platform"></a>
</p>

---

## What is Parallax?

Parallax is a Rust workspace for **honest polyglot tooling**: process-isolated Python and JavaScript workers, deterministic `.plx` snapshots, semantic-loss analysis, whole-project **Transmute** migration, continuous **Mirror** sync, **Atlas** stack detection, and **Event Horizon** for migrations that look impossible.

**Tier-1 today:** TypeScript/JavaScript → Rust project migration (`examples/weather-api`). Python ↔ JavaScript **value** migration is production-quality for bindings capture/restore. Other stacks are detected, mapped, and planned with explicit maturity — not silent fakes.

| Surface | Status |
|---|---|
| **Transmute** | TS/JS → Rust codegen path; `--require-build --require-tests` |
| **Mirror** | Linked sync + `plx sync --check` CI gate |
| **Continuum** | Same-runtime checkpoint capture (not arbitrary stack migration) |
| **Atlas** | 120+ built-in stack adapters; `plx analyze` |
| **Connectors** | 60+ language catalog; Ruby/PHP/Go workers experimental |
| **Event Horizon** | Dynamic/reflection debt; `plx impossible` |

---

## Quick start

### Prerequisites

- **Rust** 1.75+ ([rustup](https://rustup.rs/))
- **Node.js** 18+ and **Python** 3.10+ for runtime adapters
- Windows: disable Microsoft Store Python alias or install from [python.org](https://www.python.org/)

### Build

```bash
git clone https://github.com/theworker02/Parallax.git
cd parallax
cargo build -p parallax-cli --release
export PATH="$PWD/target/release:$PATH"   # optional
plx doctor
```

### Commands that show the product

```bash
# Value migration (Python ↔ JavaScript)
plx migrate examples/demo.py --to javascript -o /tmp/out.js

# Project migration (TypeScript → Rust)
plx migrate examples/weather-api --to rust -o examples/weather-api-rust --require-build --require-tests

# Stack detection
plx analyze examples/stacks/nest-prisma --to rust
plx analyze examples/weather-api --to rust

# Continuous sync
plx link examples/weather-api examples/weather-api-rust
plx sync --check

# Impossible migration analysis
plx observe examples/hostile-dynamic
plx impossible examples/hostile-dynamic --to rust
```

Global flags: `--json`, `--verbose`, `--trace`. Full reference: [CLI docs](https://parallax-runtime.github.io/parallax/cli.html).

---

## Architecture

```mermaid
flowchart TB
  CLI[plx CLI] --> RT[RuntimeManager]
  RT --> PY[Python worker]
  RT --> JS[Node worker]
  RT --> WASM[wasmtime]
  CLI --> TM[Transmute]
  TM --> AT[Atlas detect]
  TM --> PUIR[PUIR / ProjectGraph]
  CLI --> MR[Mirror sync]
  CLI --> HZ[Event Horizon]
  RT --> PIR[PIR values]
```

```text
crates/                 22 Rust workspace members (Event Horizon = one crate)
├── parallax-cli        plx / parallax binaries
├── parallax-runtime    adapter orchestration + workers
├── parallax-transmute  project migration engine
├── parallax-mirror     linked sync + semantic diff
├── parallax-atlas      120+ stack adapters + analyze
├── parallax-horizon    impossible migration analysis
├── parallax-connectors 60+ language catalog
└── parallax-ir …       PIR, PUIR, UES, protocol, snapshot
adapters/               embedded Python/JS/Ruby/PHP/Go workers
docs/                   mdBook → GitHub Pages
examples/               demos, weather-api, stack fixtures
```

Deep dive: [Architecture](https://parallax-runtime.github.io/parallax/architecture.html) · [Atlas index](https://parallax-runtime.github.io/parallax/adapters/index.html) · [Transmute](https://parallax-runtime.github.io/parallax/transmute.html) · [Mirror](https://parallax-runtime.github.io/parallax/mirror.html) · [Horizon](https://parallax-runtime.github.io/parallax/horizon.html)

---

## Crates

All workspace crates are **v0.1.0**. crates.io publication is **pending** (requires a valid API token); links below are the intended registry URLs.

| Crate | crates.io | docs.rs |
|---|---|---|
| [parallax-core](https://crates.io/crates/parallax-core) | core types, errors, versioning | [docs](https://docs.rs/parallax-core/0.1.0) |
| [parallax-adapter-sdk](https://crates.io/crates/parallax-adapter-sdk) | Atlas adapter contracts | [docs](https://docs.rs/parallax-adapter-sdk/0.1.0) |
| [parallax-diagnostics](https://crates.io/crates/parallax-diagnostics) | tracing and diagnostics | [docs](https://docs.rs/parallax-diagnostics/0.1.0) |
| [parallax-ir](https://crates.io/crates/parallax-ir) | PIR value IR | [docs](https://docs.rs/parallax-ir/0.1.0) |
| [parallax-pcir](https://crates.io/crates/parallax-pcir) | Continuation IR | [docs](https://docs.rs/parallax-pcir/0.1.0) |
| [parallax-puir](https://crates.io/crates/parallax-puir) | Universal Program IR | [docs](https://docs.rs/parallax-puir/0.1.0) |
| [parallax-security](https://crates.io/crates/parallax-security) | limits and sandbox policy | [docs](https://docs.rs/parallax-security/0.1.0) |
| [parallax-project](https://crates.io/crates/parallax-project) | project graph | [docs](https://docs.rs/parallax-project/0.1.0) |
| [parallax-protocol](https://crates.io/crates/parallax-protocol) | worker NDJSON protocol | [docs](https://docs.rs/parallax-protocol/0.1.0) |
| [parallax-snapshot](https://crates.io/crates/parallax-snapshot) | `.plx` snapshots | [docs](https://docs.rs/parallax-snapshot/0.1.0) |
| [parallax-ues](https://crates.io/crates/parallax-ues) | Universal Execution State | [docs](https://docs.rs/parallax-ues/0.1.0) |
| [parallax-migrate](https://crates.io/crates/parallax-migrate) | migration contracts | [docs](https://docs.rs/parallax-migrate/0.1.0) |
| [parallax-runtime](https://crates.io/crates/parallax-runtime) | adapter orchestration | [docs](https://docs.rs/parallax-runtime/0.1.0) |
| [parallax-transmute](https://crates.io/crates/parallax-transmute) | project migration engine | [docs](https://docs.rs/parallax-transmute/0.1.0) |
| [parallax-adapter-js](https://crates.io/crates/parallax-adapter-js) | Node.js worker adapter | [docs](https://docs.rs/parallax-adapter-js/0.1.0) |
| [parallax-adapter-python](https://crates.io/crates/parallax-adapter-python) | Python worker adapter | [docs](https://docs.rs/parallax-adapter-python/0.1.0) |
| [parallax-adapter-wasm](https://crates.io/crates/parallax-adapter-wasm) | WASM adapter | [docs](https://docs.rs/parallax-adapter-wasm/0.1.0) |
| [parallax-connectors](https://crates.io/crates/parallax-connectors) | 60+ language catalog | [docs](https://docs.rs/parallax-connectors/0.1.0) |
| [parallax-atlas](https://crates.io/crates/parallax-atlas) | stack detection & adapters | [docs](https://docs.rs/parallax-atlas/0.1.0) |
| [parallax-mirror](https://crates.io/crates/parallax-mirror) | linked sync & semantic diff | [docs](https://docs.rs/parallax-mirror/0.1.0) |
| [parallax-horizon](https://crates.io/crates/parallax-horizon) | Event Horizon analysis | [docs](https://docs.rs/parallax-horizon/0.1.0) |
| [parallax-cli](https://crates.io/crates/parallax-cli) | `plx` / `parallax` CLI | [docs](https://docs.rs/parallax-cli/0.1.0) |

Install the CLI once published:

```bash
cargo install parallax-cli
```

---

## Examples

| Path | Purpose |
|---|---|
| `examples/demo.py` / `demo.js` | Python ↔ JS value migration |
| `examples/weather-api` | Transmute + Mirror reference (TS → Rust) |
| `examples/stacks/nest-prisma` | Atlas: NestJS + Prisma detection |
| `examples/stacks/fastapi-sqlalchemy` | Atlas: FastAPI + SQLAlchemy + Ruff |
| `examples/stacks/tauri-desktop` | Atlas: Tauri desktop shell |
| `examples/hostile-dynamic` | Event Horizon dynamic/reflection fixture |
| `examples/custom-adapter` | Third-party Atlas adapter stub |

---

## Documentation

| Resource | Link |
|---|---|
| **Site** | https://parallax-runtime.github.io/parallax/ |
| **Local preview** | `cd docs && mdbook serve --open` |
| **Changelog** | [CHANGELOG.md](CHANGELOG.md) |
| **Contributing** | [CONTRIBUTING.md](CONTRIBUTING.md) |
| **Security** | [SECURITY.md](SECURITY.md) |

---

## Development

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p parallax-atlas -p parallax-horizon
cargo build -p parallax-cli
cd docs && mdbook build
```

### CI

| Workflow | Purpose |
|---|---|
| [`ci.yml`](.github/workflows/ci.yml) | fmt, clippy, cross-platform test, mdBook, stack analyze, mirror gate |
| [`pages.yml`](.github/workflows/pages.yml) | Deploy docs |
| [`release.yml`](.github/workflows/release.yml) | Release archives on `v*` tags |

---

## Status

Parallax reports **Unsupported** instead of pretending lossy or unimplemented paths succeeded. Non–Tier-1 language pairs may detect and plan but will not claim full codegen. See [Limitations](https://parallax-runtime.github.io/parallax/limitations.html).

---

## License

Licensed under the [Apache License, Version 2.0](LICENSE).

<p align="center">
  <img src="docs/assets/parallax-wordmark.svg" alt="Parallax wordmark" width="200">
</p>
