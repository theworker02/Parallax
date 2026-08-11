# Privacy policy

**Effective date:** 2026-08-11  
**Applies to:** the Parallax open-source CLI, runtime libraries, and related documentation in this repository.

This policy describes how Parallax handles data when you build or run it from this project. It is written for a **local developer tool**, not a hosted SaaS product.

## Summary

- Parallax runs **on your machine** (or in your own CI / containers).
- As of this workspace, Parallax **does not include telemetry, analytics, crash reporting, or “phone-home” network calls by default**.
- Program source, captured bindings, PIR, and `.plx` snapshots stay where you put them unless **you** transmit them.

If this changes in a future release, it will be called out in [CHANGELOG.md](CHANGELOG.md) and this file will be updated.

## What Parallax processes locally

When you invoke `plx` / `parallax` (or use the libraries), Parallax may read and write:

| Data | Why |
|---|---|
| Source files and paths you pass on the CLI | Execute, capture, migrate, restore |
| Guest stdio / captured output (subject to size limits) | Report results and errors |
| Host environment probes | `plx doctor` / adapter readiness (e.g. locating `python`, `node`) |
| PIR documents and `.plx` snapshots | Migration and persistence |
| Local logs when you enable `--verbose` / `--trace` | Diagnostics on stderr |

Guest programs you choose to run can themselves access the network or filesystem according to the **host OS user** and process environment. Parallax’s sandbox policy flags are not a complete multi-tenant jail in `0.1.x` — see [SECURITY.md](SECURITY.md) and the [security documentation](docs/src/security.md).

## Snapshots and sensitive data

`.plx` snapshots can contain **bindings and metadata from your programs** (for example names, strings, numbers, and other PIR-encoded values). Treat them like any other artifact that may include secrets or personal data:

- Do not commit snapshots that contain credentials or private information.
- Share snapshots only over channels you trust.
- Content hashing detects corruption; it is **not** encryption or access control.

## Network activity

Parallax itself is not designed to contact Parallax maintainers or third-party analytics services during normal CLI use.

Indirect network use may still occur when **you** or your environment:

- Fetch crates, toolchains, or CI dependencies
- Run guest code that opens network connections
- Browse or build documentation / GitHub Pages

## Third-party runtimes

Python, Node.js, and wasmtime are separate software with their own privacy characteristics. Parallax launches them locally as documented; it does not replace those projects’ policies.

## Children

Parallax is a developer tool and is not directed at children.

## Contact

For privacy questions about this open-source project, open a GitHub issue or discussion on the repository, or contact the maintainers at:

**privacy@parallax.dev** *(placeholder — replace with a maintained address or GitHub contact before public launch)*

Security vulnerabilities should follow [SECURITY.md](SECURITY.md), not this mailbox alone.

## Changes

Material changes to this policy will be noted in [CHANGELOG.md](CHANGELOG.md) and by updating the effective date above.
