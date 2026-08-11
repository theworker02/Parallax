# Language connectors

Parallax catalogs **dozens of languages** as first-class connectors — not only the four production runtimes.

```bash
plx connectors
plx connectors --pairs
plx connectors go
plx connectors --maturity production
plx connectors --family scripting --json
```

## What a connector is

| Role | Meaning |
|------|---------|
| **Runtime** | `RuntimeAdapter` registered with `plx runtimes` / `doctor` |
| **Value migrate** | PIR capture/restore across runtimes |
| **Transmute source** | Project analysis → PUIR |
| **Transmute target** | Codegen backend |

## Production / experimental execute today

| Connector | Execute | Value migrate | Notes |
|-----------|---------|---------------|-------|
| python | YES | YES | NDJSON worker |
| javascript | YES | YES | NDJSON worker |
| typescript | via JS | — | Analyze via tsc API |
| wasm | YES | NO | wasmtime in-process |
| **ruby** | YES (experimental) | PARTIAL | NDJSON worker |
| **php** | YES (experimental) | PARTIAL | NDJSON worker (when `php` on PATH) |
| **go** | EXPERIMENTAL | NO | NDJSON worker via `go run` |

```bash
plx run examples/demo.rb --runtime ruby --capture state
plx run examples/demo.go --runtime go
plx run examples/demo.php --runtime php --capture state
```

## Maturity (honest)

| Level | Meaning |
|-------|---------|
| **production** | Real worker/engine + tests (Python, JavaScript, WASM; TypeScript analyze) |
| **experimental** | Partial path (Ruby/PHP/Go workers; Rust Transmute target; reverse sync gated) |
| **scaffold** | Identity registered; host probed; **execute/restore return Unsupported** |
| **planned** | Catalogued for roadmap; same scaffold behavior |

Scaffold connectors exist so every serious language has a stable id, extension map, pair matrix row, and contribution hook — **not** so Parallax pretends to migrate COBOL today.

## Families covered

Systems (C, C++, Rust, Go, Zig, …), managed VM (Java, Kotlin, C#, Dart, …), scripting (Ruby, PHP, Perl, Lua, …), functional (Haskell, OCaml, Elixir, Erlang, Clojure, …), mobile (Swift, Objective-C), data science (R, Julia), shell, SQL/GraphQL, smart contracts (Solidity, Move, Cairo), HDL, and more.

See `plx connectors` for the live table (60+ entries).

## Pair highlights

```text
typescript → rust     tier1
python → rust         tier2
typescript → go       tier2
java → rust           scaffold
csharp → rust         scaffold
solidity → rust       scaffold
```

Full list: `plx connectors --pairs`.

## Contributing a real adapter

1. Pick a **scaffold** id from the catalog (`plx connectors <id>`).
2. Add `adapters/<id>/` worker speaking the NDJSON protocol **or** an in-process engine.
3. Raise maturity only when execute/capture/restore (as claimed) have tests.
4. Follow [Adapters overview](./overview.md) — never claim `YES` for unsupported ops.

Dedicated crates remain for production: `parallax-adapter-python`, `parallax-adapter-js`, `parallax-adapter-wasm`. The catalog lives in `parallax-connectors`.
