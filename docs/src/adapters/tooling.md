# Tooling adapters

Atlas detects formatters, linters, codegen inputs, and desktop GUI shells. These adapters inform stack analysis and dependency mapping — they do not auto-run external tools today.

## Formatters (`AdapterKind::Formatter`)

| Adapter | Detection signal |
|---|---|
| Prettier | `prettier` dep, `.prettierrc`, `prettier.config.js` |
| Biome | `@biomejs/biome`, `biome.json` |
| rustfmt | `Cargo.toml`, `rustfmt.toml` |
| Black | `[tool.black]` in pyproject |
| Ruff format | `[tool.ruff]` + format section |
| gofmt | `go.mod` present |
| dart format | `pubspec.yaml`, `analysis_options.yaml` |

## Linters (`AdapterKind::Linter`)

| Adapter | Detection signal |
|---|---|
| ESLint | `eslint` dep, `eslint.config.js`, `.eslintrc.*` |
| Clippy | Rust project (`Cargo.toml`) |
| Ruff lint | `[tool.ruff.lint]` |
| Pylint | `pylint` dep |
| golangci-lint | `.golangci.yml` |
| RuboCop | `rubocop` gem, `.rubocop.yml` |
| mypy | `[tool.mypy]`, `mypy.ini` |

## Codegen (`AdapterKind::Codegen`)

| Adapter | Detection signal |
|---|---|
| OpenAPI / Swagger | `openapi.yaml`, `swagger.json`, FastAPI, `@nestjs/swagger` |
| Protocol Buffers | `*.proto`, `prost`, `tonic`, `protobuf` deps |
| GraphQL Codegen | `@graphql-codegen/*`, `codegen.yml` |
| OpenAPI Generator | `openapitools.json` |

Mappings example: `@nestjs/swagger` → `utoipa` (Axum OpenAPI) with honest confidence.

## Desktop GUI (`AdapterKind::DesktopGui`)

| Adapter | Detection signal |
|---|---|
| Tauri | `@tauri-apps/api`, `src-tauri/tauri.conf.json` |
| Electron | `electron` dep, `electron-builder.yml` |
| Wails | `wails.json`, Go + frontend bundle |

Try the fixture: `plx analyze examples/stacks/tauri-desktop`.

## CLI grouping

`plx adapters` lists these under **FORMATTERS**, **LINTERS**, **CODEGEN**, and **DESKTOP GUI** — see [Adapter index](./index.md).
