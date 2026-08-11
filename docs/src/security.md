# Security & limits

Vulnerability reporting, supported versions, and a concise threat-model summary for operators: **[SECURITY.md](https://github.com/parallax-runtime/parallax/blob/main/SECURITY.md)** in the repository root.

Implemented primarily in `parallax-security` and enforced by adapters / the runtime manager.

## Threat model (0.1)

Parallax assumes **developer-trusted guest code** on a local or CI machine. It is **not** a hardened multi-tenant sandbox.

What exists today:

- Subprocess isolation for Python / JS
- Wall-clock timeouts
- Output / message size limits in `ExecutionLimits`
- WASM fuel budgets via wasmtime
- Bounded concurrent workers
- Explicit capability tokens recorded in state metadata

What does **not** exist yet:

- seccomp / Seatbelt / Windows job objects
- Network namespace isolation
- Filesystem jails
- Cryptographic attestation of snapshots

## `SandboxPolicy`

| Field | Default | Notes |
|---|---|---|
| `limits.timeout` | 30s | Wall clock |
| `limits.max_output_bytes` | 1 MiB | Stdio capture budget |
| `limits.max_message_bytes` | 16 MiB | Protocol message ceiling |
| `limits.max_memory_bytes` | 256 MiB | Soft hint where supported |
| `limits.max_fuel` | 10_000_000 | WASM |
| `allow_network` | `false` | Policy flag (not fully enforced in MVP workers) |
| `allow_fs_read` | `true` | Guests can read files the OS user can read |
| `allow_fs_write` | `false` | Policy flag |
| `max_concurrent_workers` | `4` | Manager hard limit |

`SandboxPolicy::strict()` tightens timeouts and memory for experimentation.

## Error codes worth knowing

| Code | Meaning |
|---|---|
| `CapabilityViolation` | Requested feature not available |
| `ResourceLimitExceeded` | Concurrency / size / fuel |
| `ExecutionTimeout` | Deadline exceeded |
| `AdapterCrashed` | Worker died unexpectedly |
| `InvalidSnapshot` | Tamper / schema failure |

## Handling untrusted input

If you must evaluate untrusted code:

1. Use `strict()` limits and short timeouts  
2. Run inside an external container / VM  
3. Do not pass secrets into guest globals  
4. Treat `.plx` files as untrusted data — validate, but do not assume secrecy  

## Supply chain

CI runs `cargo deny` / advisory checks when configured (see repository workflows). Pin toolchain via `rust-toolchain` / Actions `dtolnay/rust-toolchain`.
