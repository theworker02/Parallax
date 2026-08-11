# Limitations

This page is the honest list for `0.1`. Prefer reading it over marketing claims.

## Explicitly unsupported

- Migrating **arbitrary live call stacks** / instruction pointers across runtimes (Continuum does **not** claim this)
- **Cross-runtime** continuation resume (contract returns Unsupported; same-runtime explicit checkpoint is Experimental — see [Continuum](./continuum.md))
- Deterministic **replay engine** (journal schema only)
- Migrating **functions / closures** as callable values across runtimes
- Migrating **in-flight async tasks** / promises / coroutines
- WASM **binding capture or restore** / continuum
- Non-string map keys round-tripping cleanly to JS objects
- Shared object-identity graphs with cycles (bindings-first model; `ref` exists but restore is incomplete)
- Hard multi-tenant isolation (no seccomp/Seatbelt/Windows job-object enforcement yet)
- Mirror **reverse sync** for arbitrary nodes (gated; returns Unsupported unless ExactYes)
- Mirror **property-based equivalence** as a complete fuzzer (`plx verify --property` is notes / confidence only — not proof)
- Mirror **watch daemon** (`plx watch`) — not shipped
- Treating all language pairs as equally mature (see pair tiers in [Mirror](./mirror.md) and [Connectors](./adapters/connectors.md))
- Scaffold connectors (Go, Java, Ruby, C#, …): **identity + host probe only** — execute/restore/migrate return Unsupported until a real worker/codegen ships

## Semantic edge cases

| Case | Default behavior |
|---|---|
| Python `int` outside JS safe integer range | Promote to JS **BigInt** |
| Same, with `--no-prefer-bigint` | **Reject** (`MigrationRejected` / `LOSSY`) |
| Same, with `--allow-lossy` and no BigInt | Coerce to `Number` (lossy) |
| Python `tuple` → JS | Becomes `Array` (`SAFE`) |
| Python `set` → JS | Becomes `Array` (`SAFE`) |
| `bytes` → JS | `Uint8Array` (`SAFE`) |
| Unknown host types | `Unsupported` PIR node |

## Host discovery quirks

- Windows Store Python stubs are treated as unavailable
- Discovery order: `python`, `python3`, `py`, then `%LOCALAPPDATA%\Programs\Python\*\python.exe` and Program Files trees
- Node discovery: `node`, `nodejs`, then `%ProgramFiles%\nodejs\node.exe`

## Stability

- PIR schema, protocol, snapshot format, adapter interface, UES format, PCIR schema, PUIR schema, and Mirror link format are versioned independently (`1` today unless noted)
- Breaking changes will bump those constants; loaders reject mismatches
- CLI flag surface may grow; prefer `--json` for scripting
- See [Versioning](./versioning.md) for which constant to bump
