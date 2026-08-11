# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Product version (`PARALLAX_VERSION`) comes from the workspace `Cargo.toml`.
PIR schema, worker protocol, snapshot format, and adapter interface versions are
tracked independently — see [Versioning](docs/src/versioning.md).

## [Unreleased]

### Added

#### Atlas expansion (formatters, linters, codegen, desktop, frameworks)

- **Formatters:** Prettier, Biome, rustfmt, Black, Ruff format, gofmt, dart format
- **Linters:** ESLint, Clippy, Ruff lint, Pylint, golangci-lint, RuboCop, mypy
- **Codegen:** OpenAPI/Swagger, Protocol Buffers, GraphQL Codegen, OpenAPI Generator
- **Desktop GUI (experimental):** Tauri, Electron, Wails detection
- **Frameworks:** Quarkus, Micronaut, Symfony, Slim, Beego, Buffalo
- **Build:** sbt, Leiningen
- `plx adapters` grouping extended for Formatter / Linter / Codegen / DesktopGui kinds
- Expanded `DependencyMapDb` (prettier/eslint/black/ruff, Tauri/Electron, protobuf, utoipa, Quarkus, Symfony)
- Stack demo fixtures: `examples/stacks/nest-prisma`, `fastapi-sqlalchemy`, `tauri-desktop`
- Docs: [Adapter index](docs/src/adapters/index.md), [Tooling adapters](docs/src/adapters/tooling.md), richer [Architecture](docs/src/architecture.md)
- Professional README with logo (`docs/assets/parallax-logo.png`)

#### Atlas adapter expansion

- **Frameworks (beta/experimental):** Hono, Koa, Fiber, Echo, Rocket, Ktor, Vapor, Litestar, Sanic, Phoenix, Sinatra
- **Web frontends (experimental):** Vue, Svelte, Solid, Angular — compose with backend adapters
- **Build:** pnpm, Yarn, Bun, uv, Poetry, CMake, Meson
- **Tests:** Mocha, Criterion, Kotest, NUnit, XCTest, Dart test, unittest
- **ORM/DB:** Drizzle, SeaORM, GORM, Eloquent, DynamoDB
- **Deploy/CI:** Fly.io, Railway, Netlify, GitLab CI, CircleCI, AWS Lambda hints
- **CLI:** clap, commander, cobra, click, typer
- **Validation/serialization:** Zod, Pydantic, Serde (detection)
- Expanded `DependencyMapDb` equivalences for the above (honest confidence scores)
- Docs updated under [adapters/](docs/src/adapters/frameworks.md)

#### Atlas (Phase IX foundation)

- New crates `parallax-adapter-sdk` (adapter contracts, manifests, capabilities) and `parallax-atlas` (registry, stack detection, compatibility, `parallax.lock`)
- Built-in adapters across languages, frameworks, build systems, tests, databases/ORMs, deployment, runtimes, and pair profiles — with honest maturity levels
- CLI: `plx adapters` (`list` / `info` / `capabilities` / `health` / `update` / `report`), `plx analyze`, `plx stacks`, `plx mappings`, `plx compatibility`, `plx unsupported`, `plx explain-stack`, `plx adapter new|validate` (stubs)
- Expanded dependency equivalence DB (npm / PyPI / Maven / NuGet / RubyGems / Go → multi-candidate targets)
- Example third-party adapter: `examples/custom-adapter/`
- Docs under [docs/src/adapters/](docs/src/adapters/architecture.md)

#### Language connector catalog

- New crate `parallax-connectors`: 60+ language identities (systems, JVM/.NET, scripting, functional, mobile, data science, shell, SQL, smart contracts, HDL, …)
- Each connector declares roles (runtime / value-migrate / transmute source+target), family, maturity (`production` | `experimental` | `scaffold` | `planned`), extensions, and host tools
- Scaffold `RuntimeAdapter`s registered for every runtime-role language (honest Unsupported execute/restore; host PATH probe only)
- **Experimental NDJSON workers** for Ruby, PHP, and Go (`adapters/{ruby,php,go}/`); execute demos under `examples/demo.{rb,php,go}`
- CLI: `plx connectors` (`--pairs`, `--family`, `--maturity`, id lookup, `--json`)
- Transmute/Mirror pair matrix driven by the catalog (`plx connectors --pairs`)
- Docs: [Language connectors](docs/src/adapters/connectors.md)

#### Mirror (Phase VIII foundation)

- New crate `parallax-mirror`: linked projects (`.parallax-link/`), stable semantic IDs, PUIR semantic diff, incremental sync, ownership sidecars, three-way merge stubs, sync history + rollback snapshots, differential verify via target tests
- CLI: `plx link`, `plx sync` (`--check` / `--reverse` / `--patch` / `--lint`), `plx status`, `plx ci`, `plx history`, `plx rollback`, `plx explain`, `plx why`, `plx verify`
- Default policy `source-authoritative`; reverse sync returns structured Unsupported until nodes are ExactYes
- Transactional apply (pre-apply snapshot restore on build/test failure)
- Independently versioned `mirror_link_format` on `ComponentVersions`
- Docs: [Mirror](docs/src/mirror.md); CI job for weather-api link check
- Honest limits: property fuzzing / watch daemon / full bidirectional sync not claimed complete

#### Transmute (Phase VII foundation)

- New crates `parallax-puir` (Universal Program IR), `parallax-project` (ProjectGraph), `parallax-transmute` (analyze → plan → codegen → repair → report)
- TypeScript frontend via TypeScript compiler API (`adapters/typescript/analyze.mjs`)
- Dependency mapping knowledge layer + `typescript-express→rust-axum` migration pack
- `plx migrate <project> --to rust` project path with `--dry-run`, `--verify`, `--require-build`, `--require-tests`, `--min-confidence`, `--fail-on-unsupported`, `--keep`, `--target-style`
- `plx origin path:line` source-map lookup (`.plxmap.json`)
- Reference demo: `examples/weather-api` → Rust (Axum); builds and passes migrated tests
- Independently versioned `puir_schema` on `ComponentVersions`
- Docs: [Transmute](docs/src/transmute.md)

#### Continuum (Phase VI foundation)

- New crates `parallax-pcir` (Continuation IR) and `parallax-ues` (Universal Execution State, frames, safepoints, deterministic-replay hooks)
- Independently versioned `ues_format` and `pcir_schema` on `ComponentVersions` (with checkers); PIR remains value-only
- Explicit `parallax.checkpoint()` safepoint capture in Python/JS workers producing a real UES (locals/globals + resume region) — **not** arbitrary live stack migration
- Same-runtime checkpoint resume (post-checkpoint source only); cross-runtime continuation resume returns Unsupported via `MigrationContract`
- `MigrationContract` analysis pipeline in `parallax-migrate`
- CLI: `plx continuum …`, `plx capabilities --continuations`, `plx migrate --mode continuation` (honest EXPERIMENTAL/UNSUPPORTED)
- Docs: [Continuum](docs/src/continuum.md)

#### Event Horizon (Phase X foundation)

- New crate (consolidated from an initial ~20-crate scaffold): `parallax-horizon` (PVABI, semantics, behavior, IR, VCS stubs, and orchestration in one crate with `pvabi/`, `semantics/`, `behavior/`, `ir/`, `vcs/` modules)
- Semantics: preservation strategies, specialized capsules, polyglot islands, compatibility debt, metaprogram lowering
- Behavior: static project observatory, behavioral contracts, synthesis/CEGIS scaffolds (not end-to-end)
- CLI: `plx observe`, `plx impossible`, `plx dissolve`, `plx debt`, `plx detach`, `plx reconstruct`, `plx optimize-migration`, `plx explain-barrier`, `plx blame`, `plx cherry-pick`, `plx patch --example`
- Example fixture: `examples/hostile-dynamic/` (getattr, decorators, asyncio)
- Docs: [Event Horizon](docs/src/horizon.md)
- Honest limits: observatory is static heuristics; behavioral synthesis/reconstruct/semantic-git are scaffolds

### Documentation

- Expanded project governance: privacy policy, security policy, contributing guide, and code of conduct
- Added versioning notes for independently versioned surfaces (including UES/PCIR)

## [0.1.0] - 2026-08-11

Initial public workspace (`0.1.x`). Capabilities below reflect what is implemented in-tree; see [Capabilities](docs/src/capabilities.md) and [Limitations](docs/src/limitations.md) for the honest matrix.

### Added

#### Workspace and core

- Rust workspace crates: `parallax-core`, `parallax-ir`, `parallax-protocol`, `parallax-security`, `parallax-diagnostics`, `parallax-snapshot`, `parallax-migrate`, `parallax-runtime`, language adapters, and `parallax-cli`
- Version constants for product, PIR schema (`1`), worker protocol (`1`), snapshot format (`1`), and adapter interface (`1`)
- Structured errors (`Unsupported`, `MigrationRejected`, capability / limit failures) instead of silent fakes

#### Runtime adapters

- **Python** adapter: process-isolated NDJSON worker; execute, capture bindings → PIR, restore PIR bindings
- **JavaScript** (Node.js) adapter: process-isolated NDJSON worker; execute, capture, restore
- **WebAssembly** adapter: in-process wasmtime execution for zero-arg exports with fuel limits (no binding capture/restore/migrate)
- Lenient registration so missing host runtimes still appear as unavailable via `plx doctor`

#### Migration and PIR

- Language-neutral PIR documents with schema validation
- `plx migrate` Python ↔ JavaScript pipeline: capture → analyze → convert → restore
- Semantic-loss analysis (e.g. integer precision); default BigInt promotion; `--no-prefer-bigint` / `--allow-lossy` controls
- Optional migrate output of restored target-language source and/or `.plx` snapshot; `--pir-input` offline path

#### Snapshots

- Deterministic `.plx` snapshots (`PARALLAX_PLX` magic) with SHA-256 content hashing
- `plx snapshot`, `plx inspect`, `plx restore --target <runtime>`

#### CLI (`plx` / `parallax`)

- Commands: `run`, `migrate`, `snapshot`, `inspect`, `restore`, `runtimes`, `capabilities`, `doctor`, `bench`, `version`
- Global flags: `--json`, `--verbose`, `--trace`
- `plx version` reports product + PIR / protocol / snapshot versions

#### Tooling and docs

- mdBook documentation site (architecture, PIR, protocol, adapters, security, performance)
- GitHub Actions: CI (fmt, clippy, test on Linux/macOS/Windows, mdBook, cargo-deny/audit), Pages deploy, release archives on `v*` tags
- Examples: `demo.py`, `demo.js`, `demo_bigint.py`, `hello.wat`

### Known limitations

Documented in [Limitations](docs/src/limitations.md). Notably: no call-stack / continuation / async migration; WASM has no binding migration; not a hardened multi-tenant sandbox.

[Unreleased]: https://github.com/parallax-runtime/parallax/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/parallax-runtime/parallax/releases/tag/v0.1.0
