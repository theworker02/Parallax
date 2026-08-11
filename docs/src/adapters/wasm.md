# WebAssembly adapter

Crate: `parallax-adapter-wasm`  
Engine: **wasmtime** (in-process)

## What works

- Load `.wasm` bytes or `.wat` text (wasmtime `wat` feature)
- Instantiate with **fuel** enabled
- Call a **zero-argument** exported function (default name: `run`)
- Return numeric results as JSON in `ExecutionResult.value`

```bash
plx run examples/hello.wat --entry run
# runtime: wasm  success: true  value: [42]
```

## What does not work

| Feature | Status |
|---|---|
| Binding capture | Unsupported |
| PIR restore | Unsupported (`UnsupportedValue`) |
| Cross-runtime migrate | Unsupported |
| Host imports / WASI | Not wired in 0.1 |
| Multi-arg entrypoints | Rejected with a clear error |

## Limits

Fuel comes from `SandboxPolicy.limits.max_fuel` / request limits (default 10M). Traps surface as `ExecutionFailure`.

## Why include WASM now?

To prove the adapter interface and fuel-limited execution path. State migration remains a Python/JS concern until a deliberate WASM value ABI exists.
