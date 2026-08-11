# Security policy

## Supported versions

| Version | Supported |
|---|---|
| 0.1.x | Yes — current development line |
| Earlier than 0.1 | No |

Security fixes land on the maintained `0.1.x` line first. Older tags are not supported unless a release note says otherwise.

## Reporting a vulnerability

Please use **GitHub Private vulnerability reporting** or a **draft security advisory** on this repository:

https://github.com/parallax-runtime/parallax/security

Do **not** open a public issue for an exploitable defect until a fix or coordinated disclosure plan exists.

Include as much of the following as you can:

- Parallax version or git commit (`plx version --format json`)
- Host OS and runtime details (`plx doctor --json` when available)
- Minimal reproduction steps and sample inputs (redact secrets)
- Impact assessment (local code exec, snapshot forgery, DoS, etc.)
- Whether you have a suggested fix or PoC (optional)

We will acknowledge reports as maintainers are available and work toward a fix or mitigation for supported versions.

## Scope

### In scope (examples)

- Defects that let guest code break out of intended process isolation in documented configurations
- Logic bugs that cause `.plx` loaders to accept corrupted or mismatched snapshots as valid
- Memory unsafety or privilege issues in Parallax Rust crates under normal use
- Supply-chain issues in first-party release artifacts produced by this repository’s workflows

### Out of scope / expected limitations (0.1.x)

- Using Parallax as a **hardened multi-tenant SaaS sandbox** (not claimed)
- Guest programs that harm the host when run as the same OS user **without** external containment (containers / VMs)
- Policy flags such as `allow_network` / `allow_fs_write` that are recorded but **not fully OS-enforced** in MVP workers
- Social engineering, physical access, or compromised host credentials
- Vulnerabilities solely in Python, Node.js, or wasmtime themselves (report upstream when appropriate)

## Threat model summary (0.1)

**Assumption:** guest code and `.plx` files are treated as **developer-trusted** on a local or CI machine unless the operator adds external isolation.

| Surface | What exists today | Residual risk |
|---|---|---|
| Python / JS guests | Subprocess workers, timeouts, message/output size limits, concurrency caps | Same-user OS privileges; no seccomp / Seatbelt / job-object jail |
| WASM guests | In-process wasmtime with fuel budgets | Share process with the host runtime; limited feature surface |
| Snapshots (`.plx`) | Magic, format version checks, PIR validation, SHA-256 content hash | Hash ≠ signature; untrusted snapshots can still stress parsers / workers on restore |
| Capabilities | Explicit adapter declarations; unsupported ops error out | Misconfiguration or over-trusting CLI output |

**Untrusted snapshots:** validate with `plx inspect`, restore only in disposable environments, and never feed secrets into guest globals you do not trust.

Longer notes: [Security & limits](docs/src/security.md) in the mdBook.

## Preferential handling

If you believe a report involves active exploitation against third parties, say so clearly in the private report so maintainers can prioritize.

## Supply chain

CI may run `cargo deny` / advisory checks (see `.github/workflows/`). Release binaries are built from tagged commits via the repository release workflow. Prefer verifying checksums / provenance from official release artifacts when available.
