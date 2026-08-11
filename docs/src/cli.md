# CLI reference

Binaries: **`plx`** and **`parallax`** (identical).

```text
plx [GLOBAL FLAGS] <COMMAND>
```

## Global flags

| Flag | Description |
|---|---|
| `--json` | Prefer JSON on stdout for supporting commands; errors as JSON on stderr when set |
| `-v`, `--verbose` | Richer human diagnostics |
| `--trace` | Structured tracing to stderr (JSON subscriber) |

## `plx run <file>`

Execute a guest program.

| Option | Default | Description |
|---|---|---|
| `-r`, `--runtime <name>` | inferred | `python` / `javascript` / `wasm` (aliases: `py`, `js`, `node`, `wat`) |
| `--timeout-ms <n>` | `30000` | Wall-clock timeout |
| `--entry <name>` | `run` (WASM) | Exported WASM function |
| `--capture <names>` | — | Comma-separated bindings to capture after execution |

Examples:

```bash
plx run examples/demo.py
plx run examples/demo.py --capture state
plx run examples/hello.wat --entry run
plx run app.js --runtime javascript --timeout-ms 5000
```

## `plx migrate <file> --to <runtime>`

Capture → analyze → convert → restore.

| Option | Default | Description |
|---|---|---|
| `-t`, `--to <runtime>` | *required* | Target runtime |
| `-f`, `--from <runtime>` | inferred | Source runtime override |
| `--capture <names>` | `state` | Bindings to migrate |
| `--allow-lossy` | off | Permit known-lossy conversions |
| `--no-prefer-bigint` | off | Do not auto-promote unsafe ints to BigInt |
| `-o`, `--output <path>` | — | Emit target-language source preview |
| `--snapshot <path>` | — | Write migrated `.plx` |
| `--pir-input` | off | Treat `file` as a PIR JSON document (skip live capture) |

```bash
plx migrate examples/demo.py --to javascript -o out.js
plx migrate examples/demo.js --to python --json
```

## `plx snapshot <file> -o <out.plx>`

Capture bindings into a validated snapshot.

| Option | Default | Description |
|---|---|---|
| `-o`, `--output` | *required* | Output path |
| `-r`, `--runtime` | inferred | Runtime override |
| `--capture` | `state` | Binding names |
| `--label` | — | Optional label stored in the snapshot |

## `plx inspect <file.plx>`

Validate magic, format version, PIR schema, and content hash; print summary (or JSON).

## `plx restore <file.plx> --target <runtime>`

Restore snapshot bindings into a fresh worker and report restored names / types.

## `plx runtimes`

List registered adapters with readiness and host versions (production + scaffold connectors).

## `plx connectors`

Full language connector catalog (60+ languages) with maturity and transmute roles.

```bash
plx connectors
plx connectors --pairs
plx connectors java
plx connectors --maturity scaffold --family managed_vm
plx connectors --json
```

See [Language connectors](adapters/connectors.md).

## Atlas (`plx analyze` / `adapters` / …)

Modular stack detection and adapter planning. Guide: [Atlas architecture](adapters/architecture.md).

```bash
plx adapters
plx adapters info typescript
plx adapters capabilities python
plx adapters health
plx analyze examples/weather-api --to rust
plx stacks
plx mappings axios
plx compatibility python rust
plx unsupported examples/weather-api
plx explain-stack examples/weather-api --to rust
```

| Command | Notes |
|---|---|
| `adapters [list\|info\|capabilities\|health\|update\|report]` | Built-in Atlas registry |
| `analyze [path] [--to lang] [--write-lock]` | Detect stack + estimate coverage |
| `stacks` | Target stack presets |
| `mappings [query]` | Dependency equivalence DB |
| `compatibility <src> <tgt>` | Pair feature scores |
| `unsupported [path]` | Scaffold / limited adapters |
| `explain-stack [path] [--to]` | Why a target stack was chosen |
| `adapter new\|validate` | Third-party tooling stubs |

## `plx capabilities [runtime]`

Print the capability matrix (`YES` / `PARTIAL` / `EXPERIMENTAL` / `NO`).

Optional positional filter: `plx capabilities python`.

## `plx doctor`

Probe the host for Python, Node, and WASM readiness. Exit non-zero if **no** runtime is ready.

## `plx bench`

Measured micro-benchmark of capture → migrate → restore.

| Option | Default | Description |
|---|---|---|
| `--iterations <n>` | `5` | Sample count |
| `--file <path>` | `examples/demo.py` | Source program |
| `--to <runtime>` | `javascript` | Target |

```bash
plx bench --iterations 20 --json
```

## Mirror (`plx link` / `sync` / …)

Continuous sync after a Transmute migration. Full guide: [Mirror](mirror.md).

```bash
plx link ./api-ts ./api-rust
plx sync
plx sync --check
plx status --json
plx ci
plx history
plx rollback
plx explain src/service.rs:20 -C ./api-rust
plx why src/service.rs -C ./api-rust
plx verify
```

| Command | Notes |
|---|---|
| `link <src> <tgt> [--policy …]` | Default policy: `source-authoritative` |
| `sync [--check\|--reverse\|--patch\|--lint\|--no-verify]` | Incremental; `--check` is non-mutating |
| `ci` | `sync --check` + differential verify |
| `status` | Drift; `--json` for editors/CI |

## `plx version`

Print product / schema versions (`--format json` or global `--json`).
