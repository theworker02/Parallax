# Continuum (Phase VI)

Continuum moves beyond **value/state** PIR exchange toward suspended **computation** migration:

**pause → capture continuation → Universal Execution State → translate → resume**

This chapter describes what is implemented today versus what remains experimental or unsupported.

## PIR vs UES vs PCIR

| Artifact | Models | Version constant |
|---|---|---|
| **PIR** | Portable **values** / object graphs | `pir_schema` |
| **UES** | Suspended **execution** (control, frames, heap, capabilities) | `ues_format` |
| **PCIR** | Portable **control-flow** subset for supported regions | `pcir_schema` |

These versions advance independently (see [Versioning](./versioning.md)). Serialization alone is **not** migration.

## What is real in this milestone

1. **Types + serde** for `UniversalExecutionState`, `UniversalFrame`, PCIR ops, binary/JSON envelopes, version rejection.
2. **Safepoint model** with machine-readable reports (`can_capture` / `snapshot` / `replay` / `migrate`, targets, semantic loss).
3. **Explicit checkpoint capture** in Python and JavaScript workers via `parallax.checkpoint(label)` (and `@parallax.safepoint` / `parallax.safepoint` conceptually).
4. **Same-runtime resume** of the **post-checkpoint** source region with restored bindings (not a full program restart).
5. **`MigrationContract` analysis** before continuation attempts; clear reject reports when unsatisfied.
6. **Continuation capability matrix** via CLI.

## What is Explicitly Unsupported / Experimental

| Capability | Status |
|---|---|
| Arbitrary live stack frame migration | **NO** — not claimed |
| Cross-runtime continuation resume | **NO** (contract-gated) |
| Deterministic replay engine | **UNSUPPORTED** (journal schema / hooks only) |
| Async / await / yield migration | **NO** |
| WASM continuum | **NO** |
| Same-runtime checkpoint capture + resume | **EXPERIMENTAL** |

If a path is not truly implemented, Continuum returns structured `Unsupported` / capability levels (`YES` / `PARTIAL` / `EXPERIMENTAL` / `NO`) — it never pretends resume worked.

## Safepoints

Supported boundary for this pass: **explicit checkpoint**.

```python
x = 1
parallax.checkpoint("after_init")
x = x + 41  # runs only on resume
```

At the safepoint the worker reports whether it can capture / snapshot / replay / migrate, candidate targets, and semantic-loss notes.

## CLI

```bash
# Continuation capability matrix
plx capabilities python --continuations
plx capabilities --continuations --json

# Capture UES at checkpoint (experimental)
plx continuum examples/checkpoint_demo.py -o demo.ues.json --json

# Same-runtime resume after capture
plx continuum examples/checkpoint_demo.py --resume --json

# Inspect a written UES
plx continuum demo.ues.json --inspect-ues

# Contract-only analysis
plx continuum examples/checkpoint_demo.py --analyze-only -t javascript

# Honest continuation migrate mode (rejects cross-runtime)
plx migrate examples/checkpoint_demo.py -t javascript --mode continuation
```

Value/state PIR migration remains the default:

```bash
plx migrate examples/demo.py -t javascript
```

## Migration contracts

Before a live continuation attempt, Parallax builds a `MigrationContract` describing required surviving semantics (values, locals, control position, stack frames, same-runtime vs cross-runtime resume, …). Analysis runs first; unsatisfied contracts produce a readable reject report.

## Crates

- `parallax-pcir` — Continuation IR ops / programs
- `parallax-ues` — UES, frames, safepoints, deterministic hooks, continuation matrix
- `parallax-migrate::contract` — `MigrationContract` + analysis

## Related

- [Capabilities](./capabilities.md)
- [Migration](./migration.md)
- [PIR](./pir.md)
- [Limitations](./limitations.md)
