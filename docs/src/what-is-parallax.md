# What Parallax is

Parallax is infrastructure for **polyglot execution and state migration**.

You write (or already have) a small program that defines data bindings. Parallax:

1. Executes it in a host runtime (Python, JavaScript, or WASM)
2. Captures named bindings into **PIR** (Parallax Intermediate Representation)
3. Analyzes semantic compatibility for a target runtime
4. Converts the PIR under an explicit policy
5. Restores the bindings into the target runtime
6. Optionally emits a source preview and/or a `.plx` snapshot

The headline path in `0.1` is:

```text
Python program defining `state` → PIR → JavaScript object
JavaScript program defining `state` → PIR → Python dict
```

## What Parallax is not

Be explicit about non-goals so expectations stay honest:

| Not this | Reality in 0.1 |
|---|---|
| A full cross-language VM that migrates running stacks | Stack / continuation migration is **not** supported |
| Transparent function / closure migration | Functions become `Unsupported` |
| A language transpiler | Emit is a **preview** of restored bindings, not a compiler |
| A sandbox for untrusted multi-tenant SaaS | Limits exist; network/FS isolation is policy-level, not a hardened jail |
| A drop-in replacement for gRPC / FFI | Different problem — Parallax moves **values**, not RPCs |

## Who it is for

- Runtime / compiler engineers exploring polyglot state interchange
- Tooling authors who need a checked, language-neutral value layer
- Projects that want **opt-in** migration with loss reporting instead of silent coercion

## Version posture

`0.1.0` is an early but **working** release of the migration path. APIs and PIR schema may evolve; schema and protocol versions are versioned independently (`PIR_SCHEMA_VERSION`, `PROTOCOL_VERSION`, `SNAPSHOT_FORMAT_VERSION`).
