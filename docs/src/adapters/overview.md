# Adapters overview

An adapter implements `RuntimeAdapter` in `parallax-runtime`:

- `probe` — host readiness
- `execute` — run a `ProgramSource`
- `restore` — materialize a `PirDocument`
- `capabilities` / `metadata` — declarations for CLI and snapshots

**Atlas** (Phase IX) adds a second adapter surface for *project migration*: language/framework/build/test/ORM/deploy detectors and planners via `parallax-adapter-sdk` + `parallax-atlas`. Runtime adapters and Atlas adapters are complementary — see [Atlas architecture](./architecture.md).

## Registration

The CLI registers adapters at startup:

```text
parallax_adapter_python::register_lenient
parallax_adapter_js::register_lenient
parallax_adapter_wasm::register_lenient
parallax_connectors::register_all_lenient   # 60+ language scaffolds
```

Lenient registration means a missing host binary still registers an adapter that **probes UNAVAILABLE** — `plx doctor` stays informative.

Scaffold connectors (Go, Java, Ruby, C#, …) register with honest **Unsupported** execute/restore until a real worker exists. Browse them with `plx connectors`. See [Language connectors](./connectors.md).

## Program sources

| Variant | Use |
|---|---|
| `File` | Path on disk |
| `Inline` | Source text + filename hint |
| `CaptureBindings` | Source + explicit capture names (used internally by migrate/snapshot) |
| `Bytes` | Raw WASM module bytes |

## Chapters

- [Atlas architecture](./architecture.md) — modular adapter orchestration
- [Adapter SDK](./sdk.md)
- [Language connectors](./connectors.md) — full catalog
- [Python](./python.md)
- [JavaScript](./javascript.md)
- [WebAssembly](./wasm.md)
