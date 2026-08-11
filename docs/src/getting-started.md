# Getting started

## Prerequisites

| Dependency | Required for | Notes |
|---|---|---|
| Rust **1.75+** (stable) | Build / CLI | [rustup](https://rustup.rs/) |
| **Node.js** 18+ | JavaScript adapter | Detected as `node` |
| **Python** 3.10+ | Python adapter | Detected as `python`, `python3`, `py`, then common install paths |
| Git | Clone | Optional if you use a release binary later |

> **Windows tip:** The Microsoft Store `python` alias often fails. Install from [python.org](https://www.python.org/) or ensure a real interpreter is on PATH. `plx doctor` will say clearly when Python is missing.

## Install from source

```bash
git clone https://github.com/parallax-runtime/parallax.git
cd parallax
cargo build -p parallax-cli --release
```

Binaries land at:

```text
target/release/plx
target/release/parallax
```

`plx` and `parallax` are equivalent entry points.

Add the release directory to your `PATH`, or invoke via Cargo:

```bash
cargo run -p parallax-cli --bin plx -- doctor
```

## First health check

```bash
plx doctor
```

Example healthy output (versions will vary):

```text
Parallax 0.1.0
Host: windows x86_64

[javascript] READY
  binary: node
  version: 24.x

[python] READY
  binary: .../python.exe
  version: 3.12.x

[wasm] READY
  binary: wasmtime (in-process)

Doctor: OK — at least one runtime is ready.
```

JSON form:

```bash
plx doctor --json
```

## Run a program

```bash
plx run examples/demo.py
plx run examples/demo.js
plx run examples/hello.wat --entry run
```

Runtime is inferred from the file extension (`.py`, `.js`, `.wat` / `.wasm`) unless you pass `--runtime`.

## Next steps

- [Migration demo](./migration-demo.md)
- [CLI reference](./cli.md)
- [Capabilities matrix](./capabilities.md)
