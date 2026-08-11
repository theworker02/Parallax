# Migration engine

Implemented in `parallax-migrate`.

## Goals

- Move **data bindings** between runtimes through PIR
- Detect semantic incompatibilities **before** pretending success
- Keep policy explicit (`ConversionPolicy` / CLI flags)
- Report **measured** phase timings

## Loss taxonomy

| Level | Meaning | Default policy |
|---|---|---|
| `NONE` | Equivalent | Allow |
| `SAFE` | Representation differs, semantics preserved | Allow |
| `POTENTIALLY_LOSSY` | Depends on contents | Allow (`allow_potentially_lossy`) |
| `LOSSY` | Known corruption risk (e.g. unsafe int → Number) | **Reject** unless `--allow-lossy` |
| `UNSUPPORTED` | Cannot represent | Keep as `Unsupported` node (or reject if configured) |

## Conversion policy knobs

| Field / flag | Default | Effect |
|---|---|---|
| `prefer_bigint` / *(default on)* | `true` | Unsafe ints → PIR `bigint` for JS |
| `--no-prefer-bigint` | — | Disable BigInt promotion |
| `--allow-lossy` | off | Permit `LOSSY` coercions |
| `allow_potentially_lossy` | `true` | Allow amber findings |
| `reject_unsupported` | `false` | Hard-fail on `Unsupported` |

## Phase timings

`MigrationReport.timings` fields (microseconds):

| Field | Source |
|---|---|
| `capture_us` | Live adapter execution (when used) |
| `analyze_us` | Semantic walk |
| `convert_us` | PIR rewrite |
| `restore_us` | Target adapter restore |
| `total_us` | Sum of measured phases |

Never fabricated — if a phase did not run, the optional field is omitted / zero as documented by the CLI JSON schema.

## Typical findings

- `SAFE` — tuple/set → JS array; BigInt promotion path
- `LOSSY` — integer outside `[−2^53+1, 2^53−1]` without BigInt preference
- `UNSUPPORTED` — functions, host objects without encoders

## API surface (library)

```rust
use parallax_migrate::migrate_document;
use parallax_core::{ConversionPolicy, RuntimeKind};

let (pir_out, report) = migrate_document(
    RuntimeKind::Python,
    RuntimeKind::JavaScript,
    &pir_in,
    &ConversionPolicy::default(),
)?;
```

CLI users should prefer `plx migrate` — it wires capture and restore around this function.
