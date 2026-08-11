# Changelog

User-facing changes are recorded in the repository root:

**[CHANGELOG.md](https://github.com/parallax-runtime/parallax/blob/main/CHANGELOG.md)**

The project follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and [Semantic Versioning](https://semver.org/) for the Parallax product line. Independently versioned surfaces (PIR schema, protocol, snapshot format, adapter interface) are described in [Versioning](./versioning.md).

## Snapshot of 0.1.0

Initial public workspace. Highlights:

- Rust workspace: core, PIR, protocol, security, diagnostics, snapshot, migrate, runtime, adapters, CLI
- Python and JavaScript NDJSON workers with execute / capture / restore
- WASM execution via wasmtime (zero-arg exports, fuel); no binding migration
- `plx migrate` Python ↔ JavaScript with semantic-loss analysis
- `.plx` snapshots with content hashing
- `plx doctor`, `runtimes`, `capabilities`, `bench`, `--json`
- mdBook site and GitHub Actions CI / Pages / release scaffolding

Limitations: [Limitations](./limitations.md).
