# Introduction

<div class="plx-hero">

**Parallax** is a universal polyglot execution runtime written in Rust.

Capture program state in one language, encode it as language-neutral **PIR**, and restore it in another — with real workers, measured timings, and honest semantic-loss reporting.

</div>

This site documents **what exists today** in the `0.1.x` workspace: Python ↔ JavaScript state migration, snapshots, the `plx` CLI, and a constrained WASM executor.

| Resource | Link |
|---|---|
| Source | [github.com/parallax-runtime/parallax](https://github.com/parallax-runtime/parallax) |
| Docs (this site) | [parallax-runtime.github.io/parallax](https://parallax-runtime.github.io/parallax/) |
| Changelog | [CHANGELOG.md](https://github.com/parallax-runtime/parallax/blob/main/CHANGELOG.md) |
| Security | [SECURITY.md](https://github.com/parallax-runtime/parallax/blob/main/SECURITY.md) |
| Privacy | [PRIVACY.md](https://github.com/parallax-runtime/parallax/blob/main/PRIVACY.md) |
| License | Apache-2.0 |
| MSRV | Rust 1.75 |

## Start here

1. [What Parallax is](./what-is-parallax.md) — scope and non-goals
2. [Getting started](./getting-started.md) — install and first commands
3. [Migration demo](./migration-demo.md) — Python → JavaScript in one command
4. [Architecture](./architecture.md) — crates, workers, pipeline

## Design principles

- **No fake demos** — unsupported work returns structured `Unsupported` / `MigrationRejected`
- **Measured timings** — capture / analyze / convert / restore durations are real wall-clock microseconds
- **Explicit capabilities** — every adapter declares what it can and cannot do
- **Process isolation** — Python and JavaScript guests run in subprocess workers over a versioned NDJSON protocol
