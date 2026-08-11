# Atlas adapter index

Parallax Atlas ships **120+ built-in adapters** across languages, frameworks, tooling, and deployment surfaces. Maturity is honest: `stable` / `beta` / `experimental` / `scaffold` / `parse-only`.

Browse live inventory:

```bash
plx adapters              # grouped by kind
plx adapters --json       # machine-readable
plx adapters info <id>    # manifest + capabilities
plx analyze <path>        # detect stack for a project
```

## Adapter kinds

| Kind | Examples | CLI group |
|---|---|---|
| Source / target language | TypeScript, Python, Rust, Go | `LANGUAGE ADAPTERS` / `TARGET ADAPTERS` |
| Framework | Express, FastAPI, NestJS, Axum, Quarkus | `FRAMEWORK ADAPTERS` |
| Web frontend | React, Vue, Svelte, Angular | `WEB FRONTEND` |
| Build system | npm, Cargo, pnpm, uv, Poetry, sbt | `BUILD ADAPTERS` |
| Test framework | Jest, Vitest, pytest, cargo test | `TEST ADAPTERS` |
| Database / ORM | PostgreSQL, Prisma, SQLAlchemy, SeaORM | `DATABASE` / `ORM ADAPTERS` |
| Deployment / CI | Docker, Fly.io, GitHub Actions, Lambda | `DEPLOYMENT` |
| Runtime | Node, CPython, WASM | `RUNTIME` |
| CLI framework | clap, commander, click, typer | `CLI FRAMEWORKS` |
| Validation / serialization | Zod, Pydantic, Serde | `VALIDATION` / `SERIALIZATION` |
| **Formatter** | Prettier, rustfmt, Black, Ruff, Biome | `FORMATTERS` |
| **Linter** | ESLint, Clippy, Ruff, mypy, golangci-lint | `LINTERS` |
| **Codegen** | OpenAPI/Swagger, Protobuf, GraphQL Codegen | `CODEGEN` |
| **Desktop GUI** | Tauri, Electron, Wails | `DESKTOP GUI` |
| Pair profile | TypeScript→Rust, Python→Rust | `PAIR PROFILES` |

## Stack demo fixtures

Under `examples/stacks/`:

| Fixture | Stack signal |
|---|---|
| `nest-prisma/` | NestJS + Prisma + Vitest + ESLint/Prettier |
| `fastapi-sqlalchemy/` | FastAPI + SQLAlchemy + Ruff + mypy |
| `tauri-desktop/` | Tauri desktop shell + TypeScript |

```bash
plx analyze examples/stacks/nest-prisma --to rust
plx analyze examples/stacks/fastapi-sqlalchemy --to rust
plx analyze examples/stacks/tauri-desktop
```

## Dependency mappings

Transmute uses `DependencyMapDb` for npm / PyPI / Maven / Go → multi-target equivalences:

```bash
plx mappings express
plx mappings prisma
plx mappings @tauri-apps/api
```

See [Dependencies](./dependencies.md) for confidence scoring and manual-review rules.

## Related chapters

- [Overview](./overview.md) — runtime vs Atlas adapters
- [Atlas architecture](./architecture.md)
- [Frameworks](./frameworks.md)
- [Language connectors](./connectors.md) — 60+ runtime identities (separate catalog)
