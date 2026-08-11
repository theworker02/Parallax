# Adapter security

Third-party adapters must not get unrestricted host access.

## Permissions (`AdapterPermissions`)

| Flag | Meaning |
|------|---------|
| `read_project` | Read source tree |
| `write_output` | Write generated files |
| `execute_build` | Run build/test tools |
| `network` | Network I/O |
| `read_environment` | Read env vars |

Built-ins use a fuller permission set. External adapters default to **minimal** (`read_project` only) once package loading lands.

## Isolation goals

- Crash isolation (`ADAPTER_FAILURE` without killing the migration)
- Timeouts and memory budgets for untrusted adapters
- Deterministic hooks only

Telemetry from `plx adapters report` is **local-only** by default.
