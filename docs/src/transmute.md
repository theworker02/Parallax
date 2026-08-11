# Transmute — project migration

**Transmute** migrates an entire software project from one language/ecosystem to another using a semantic pipeline — not file-by-file text translation.

```text
SOURCE PROJECT
      ↓
Project Analysis → ProjectGraph + PUIR
      ↓
Migration plan (deps, frameworks, layout)
      ↓
Code generation
      ↓
Build → Test → Repair → Report
      ↓
TARGET PROJECT
```

## First supported path

| Source | Target | Status |
|--------|--------|--------|
| TypeScript / JavaScript (Node, Express) | Rust (Axum) | **Implemented** (weather-api demo) |
| Python | Rust | Analysis / planning only |
| Other pairs | — | Structured `Unsupported` |

## CLI

```bash
# Analyze + migrate a project
plx migrate ./examples/weather-api --to rust --output ./examples/weather-api-rust --verify

# Plan only
plx migrate ./api --to rust --dry-run

# Quality gates
plx migrate ./api --to rust --require-build --require-tests --min-confidence 0.9

# Origin lookup (source maps)
plx origin src/service.rs:14 -C ./examples/weather-api-rust
```

`plx migrate` auto-selects **Transmute** when the path is a project directory (or `--to` is a language such as `rust`). Use `--mode value` for PIR state migration of a single guest program.

## Representations

| IR | Meaning |
|----|---------|
| **PIR** | Values / heap bindings |
| **PUIR** | Program semantics (functions, types, intent ops) |
| **PCIR / UES** | Suspended execution (Continuum) |
| **ProjectGraph** | Modules, deps, edges, entrypoints, tests |

PUIR is versioned independently (`puir_schema` on `plx version`).

## Workspace

Migrations write `.parallax/` under the source root (`project.json`, `graph.json`, `puir.json`, `plan.json`, …) for inspection and future incremental updates.

## Demo

```bash
plx migrate examples/weather-api --to rust -o examples/weather-api-rust --require-build --require-tests
cd examples/weather-api-rust
cargo test
cargo run
```

See `PARALLAX_MIGRATION.md` and `parallax-report.json` in the output directory.

## Honesty rules

- Compatibility percentages come from measured analysis (PUIR coverage, dep confidence, …) — never invented.
- Uncertain regions get `// PARALLAX REVIEW:` markers and report entries.
- Secrets in `.env` are never copied; only `.env.example` names are emitted.
- Cross-language project migration is **not** Continuum live-stack resume.
