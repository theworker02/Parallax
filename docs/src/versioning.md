# Versioning

Parallax uses **several version numbers** that can advance independently. Bumping the product version does not automatically imply a PIR or protocol break, and vice versa.

Inspect live values:

```bash
plx version
plx version --format json
```

Constants live in `parallax-core` (`version.rs`):

| Surface | Constant | Role |
|---|---|---|
| **Parallax (product)** | `PARALLAX_VERSION` | SemVer from workspace `Cargo.toml` (`0.1.x` today). CLI, crates, and release tags. |
| **PIR schema** | `PIR_SCHEMA_VERSION` | Language-neutral IR document schema. Loaders reject unsupported schema numbers. |
| **Worker protocol** | `PROTOCOL_VERSION` | NDJSON envelope version between host adapters and Python/JS workers. |
| **Snapshot format** | `SNAPSHOT_FORMAT_VERSION` | `.plx` container fields / hashing contract. |
| **Adapter interface** | `ADAPTER_INTERFACE_VERSION` | Host-facing adapter metadata / registration contract. |
| **UES format** | `UES_FORMAT_VERSION` | Universal Execution State wire format (execution, not values). |
| **PCIR schema** | `PCIR_SCHEMA_VERSION` | Continuation IR schema for supported control regions. |
| **PUIR schema** | `PUIR_SCHEMA_VERSION` | Program / project IR used by Transmute and Mirror. |
| **Mirror link format** | `MIRROR_LINK_FORMAT_VERSION` | `.parallax-link/` metadata layout. |

As of `0.1.0`, these integer surfaces are at **`1`** unless noted otherwise.

## Compatibility expectations

- **Product SemVer** (`CHANGELOG.md`): user-facing CLI and library behavior for the Parallax release line.
- **Integer schema/protocol/format versions**: treat a bump as a potential **breaking** change for that surface. Readers should reject unknown or mismatched versions rather than guessing.
- **Adapters**: workers and host crates must agree on `PROTOCOL_VERSION`. Capability matrices may grow without a protocol bump when messages stay compatible; incompatible message shapes require a protocol bump.
- **Snapshots**: `plx inspect` / restore validate magic, `format_version`, PIR, and content hash. Older writers are not guaranteed to load in newer readers until migration rules are documented.

## When to bump what

| Change | Bump |
|---|---|
| CLI flag, migrate policy default, crate API for users | Product SemVer (per [SemVer](https://semver.org/) once published; pre-1.0 may move faster) |
| PIR node shapes or document required fields | `PIR_SCHEMA_VERSION` |
| NDJSON request/response envelope or required fields | `PROTOCOL_VERSION` |
| `.plx` top-level fields or hash canonicalization | `SNAPSHOT_FORMAT_VERSION` |
| `RuntimeAdapter` method/metadata contract across crates | `ADAPTER_INTERFACE_VERSION` |
| UES document fields / envelope | `UES_FORMAT_VERSION` |
| PCIR op set or program schema | `PCIR_SCHEMA_VERSION` |
| PUIR item / program schema | `PUIR_SCHEMA_VERSION` |
| `.parallax-link/` layout | `MIRROR_LINK_FORMAT_VERSION` |

Record product-facing changes in the root [CHANGELOG.md](../../CHANGELOG.md). Call out schema/protocol/format bumps explicitly in the same release notes.

## Related chapters

- [PIR](./pir.md)
- [Continuum](./continuum.md)
- [Worker protocol](./protocol.md)
- [Snapshots](./snapshots.md)
- [Changelog](./changelog.md)
