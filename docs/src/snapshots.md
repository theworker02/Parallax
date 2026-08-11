# Snapshots (`.plx`)

Deterministic JSON documents with integrity hashing. Implemented in `parallax-snapshot`.

## Format

| Field | Description |
|---|---|
| `magic` | Must be `PARALLAX_PLX` |
| `format_version` | `SNAPSHOT_FORMAT_VERSION` (`1`) |
| `id` | UUID |
| `created_at` | UTC timestamp |
| `runtime` | Origin / target runtime kind |
| `label` | Optional |
| `state` | `ExecutionState` shell (capabilities, heap JSON, metadata) |
| `pir` | Full `PirDocument` |
| `content_hash` | SHA-256 hex of canonical `{format_version, runtime, state, pir}` |

Loaders reject bad magic, unsupported versions, invalid PIR, and hash mismatches (`InvalidSnapshot`).

## CLI

```bash
# Capture
plx snapshot examples/demo.py -o demo.plx --label demo

# Inspect
plx inspect demo.plx
plx inspect demo.plx --json

# Restore into a runtime
plx restore demo.plx --target javascript
```

During migrate:

```bash
plx migrate examples/demo.py --to javascript --snapshot migrated.plx -o migrated.js
```

## What is stored

For the supported binding-capture path, the important payload is **`pir.bindings`**. Stack frames, instruction pointers, and async state are typically empty — capabilities say so.

## Integrity model

Snapshots are not cryptographic signatures. The content hash detects accidental corruption and casual edits. Treat them like build artifacts: transfer over trusted channels if the binding data is sensitive.
