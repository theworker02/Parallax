# Contributing to Parallax

Thanks for helping build Parallax.

Longer narrative (architecture context, same norms):  
**[Documentation → Contributing](https://parallax-runtime.github.io/parallax/contributing.html)**

Please also read the [Code of Conduct](CODE_OF_CONDUCT.md).

## Principles

- **No vaporware** — document and demo only what the code does. Unsupported work returns structured `Unsupported` / `MigrationRejected` (or related) errors, never silent fakes.
- Prefer small, focused pull requests.
- Keep README, mdBook, and CLI help aligned with real behavior.
- Add tests for PIR, migration, snapshot, and adapter critical paths when you change them.

## Development setup

### Prerequisites

- **Rust** 1.75+ (stable) — see `rust-toolchain.toml` / workspace `rust-version`
- **Node.js** 18+ on `PATH` (JavaScript adapter)
- **Python** 3.10+ discoverable as `python` / `python3` / `py` (Python adapter)
- Optional: [mdBook](https://rust-lang.github.io/mdBook/) for docs (`cargo install mdbook`)
- Optional: `cargo-deny` for dependency policy checks

```bash
git clone https://github.com/parallax-runtime/parallax.git
cd parallax
cargo build --workspace
cargo test --workspace
cargo run -p parallax-cli --bin plx -- doctor
```

Release CLI binary:

```bash
cargo build -p parallax-cli --release
# target/release/plx  (and parallax)
```

### Docs site

```bash
cd docs
mdbook serve --open   # http://localhost:3000
mdbook build          # → docs/book/
```

## Checks before opening a PR

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check          # if cargo-deny is installed
cd docs && mdbook build
```

CI (`.github/workflows/ci.yml`) runs fmt, clippy, and tests on Linux, macOS, and Windows, plus docs and audit-related jobs.

## Adapter conformance

Adapters live under `crates/parallax-adapter-*` with worker scripts in `adapters/`. They register with the runtime manager and should:

1. Implement the `RuntimeAdapter` surface used by `parallax-runtime` (`probe`, `execute`, `restore`, `capabilities` / `metadata` as applicable).
2. Declare capabilities **honestly** — if capture/restore/migrate is unsupported, return structured errors (WASM binding migration is the reference “no” path).
3. Speak the versioned NDJSON worker protocol when using subprocess workers (`PROTOCOL_VERSION` in `parallax-core` / `parallax-protocol`). Reject mismatched protocol versions.
4. Prefer **lenient registration**: missing host binaries still register and probe as unavailable so `plx doctor` stays useful.
5. Respect `ExecutionLimits` / sandbox policy fields that are enforced today (timeouts, message/output sizes, WASM fuel, concurrency). Do not claim OS-level network/FS jails that are not implemented.
6. Keep worker scripts and Rust adapter crates in sync; add tests for execute / capture / restore paths you touch.

See [Adapters overview](docs/src/adapters/overview.md) and [Versioning](docs/src/versioning.md).

## Pull request guidelines

- Describe **what** changed and **why**; link issues when relevant.
- Update docs/CHANGELOG for user-visible behavior.
- Do not expand the capabilities matrix in docs without matching code and tests.
- Keep diffs focused; avoid drive-by reformatting of unrelated crates (other work may be in flight).
- Ensure CI is green (or explain failures that are environmental).

## Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

| Prefix | Use |
|---|---|
| `feat:` | User-visible feature |
| `fix:` | Bug fix |
| `docs:` | Documentation only |
| `test:` | Tests only |
| `refactor:` | Internal change without behavior change |
| `chore:` | Maintenance, deps, tooling |
| `ci:` | CI configuration |

Examples: `feat: prefer BigInt for oversized Python ints`, `docs: expand SECURITY threat model`.

## License and DCO

Contributions are accepted under the [Apache License, Version 2.0](LICENSE).

By contributing, you indicate that you have the right to submit the work and that it is provided under Apache-2.0. A formal CLA is **not** required at this time. If the project later adopts a CLA or a signed DCO (`Signed-off-by:`), maintainers will announce it in the changelog and this guide.

## Security and privacy

- Vulnerabilities: [SECURITY.md](SECURITY.md) (private reporting).
- Privacy expectations for the local CLI: [PRIVACY.md](PRIVACY.md).
- Product versioning surfaces: [docs/src/versioning.md](docs/src/versioning.md).

## Project layout (quick map)

```text
crates/           Rust workspace
adapters/         Python & JS worker scripts
docs/             mdBook source
examples/         demos
.github/workflows CI, Pages, release
```

Questions that are not security-sensitive: open a GitHub issue or discussion on the repository.
