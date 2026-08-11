# Capabilities matrix

Adapters declare capabilities explicitly. Attempting an unsupported operation returns a structured error — it is never silently ignored.

Run `plx capabilities` on your machine for the live table.

## Python

| Capability | Level |
|---|---|
| Values | YES |
| Globals | YES |
| Locals | PARTIAL |
| Functions | PARTIAL (encoded as function / unsupported for migrate) |
| Closures | PARTIAL |
| Stack capture | EXPERIMENTAL |
| Stack frames / control position | EXPERIMENTAL (explicit checkpoint only) |
| Continuation capture / restore | EXPERIMENTAL (same-runtime checkpoint) |
| Cross-runtime resume | NO |
| Async migration | NO |
| Execution | YES |
| Stdio capture | YES |
| Timeouts | YES |
| Resource limits | PARTIAL |
| Cancellation | YES |

## JavaScript (Node.js)

| Capability | Level |
|---|---|
| Values | YES |
| Globals | YES |
| Locals | PARTIAL |
| Functions | PARTIAL |
| Closures | PARTIAL |
| Stack capture | NO |
| Stack frames / control position | EXPERIMENTAL (explicit checkpoint only) |
| Continuation capture / restore | EXPERIMENTAL (same-runtime checkpoint) |
| Cross-runtime resume | NO |
| Async migration | NO |
| Execution | YES |
| Stdio capture | YES |
| Timeouts | YES |
| Resource limits | PARTIAL |
| Cancellation | YES |

## WebAssembly (wasmtime)

| Capability | Level |
|---|---|
| Values | PARTIAL |
| Globals / locals / closures | NO |
| Functions | PARTIAL (call zero-arg exports) |
| Stack / continuation / async | NO |
| Execution | YES |
| Stdio capture | NO |
| Timeouts / fuel / limits | YES |
| Cancellation | YES |
| State restore / migrate | **NO** |

## Cross-runtime migration

| From \\ To | Python | JavaScript | WASM |
|---|---|---|---|
| **Python** | restore OK | **migrate OK** | Unsupported |
| **JavaScript** | **migrate OK** | restore OK | Unsupported |
| **WASM** | Unsupported | Unsupported | Unsupported |

## Language connectors

Run `plx connectors` for the live catalog. Summary:

| Maturity | Count (approx.) | Execute / migrate |
|---|---|---|
| production | 4 (py, js, ts analyze, wasm) | Real (wasm: execute only) |
| experimental | ruby, php, go (+ Rust target) | Workers when host present |
| scaffold / planned | 50+ | **NO** (registered Unsupported) |

Scaffold adapters appear in `plx runtimes` as DEGRADED (host found) or UNAVAILABLE (host missing). That is intentional.

## Mirror (project sync)

| Capability | Level |
|---|---|
| Link TS/JS → Rust | YES (Tier 1 demo) |
| Semantic diff + incremental sync | YES |
| `sync --check` / `plx ci` | YES |
| Manual-region preservation (sidecar) | PARTIAL |
| Three-way semantic merge | PARTIAL (conflicts reported; no silent guess) |
| Differential execution | PARTIAL (migrated test suite; not formal proof) |
| Reverse sync | EXPERIMENTAL / gated Unsupported |
| Watch daemon | NO |
| Property fuzz equivalence | EXPERIMENTAL notes only |
