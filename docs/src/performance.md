# Performance

Parallax optimizes for **correct, measurable migration**, not micro-benchmark theater.

## How to measure

```bash
# End-to-end averages (real samples)
plx bench --iterations 20 --json

# Single migration with phase breakdown
plx migrate examples/demo.py --to javascript --json
```

JSON includes per-phase microseconds. Use those — do not invent numbers for blog posts.

## Cost model (qualitative)

| Phase | Dominant cost |
|---|---|
| Capture | Process spawn + interpreter startup + encode |
| Analyze / convert | Usually tiny vs spawn for demo-sized graphs |
| Restore | Process spawn + decode |
| WASM execute | In-process; fuel accounting overhead |

On warm disks, demo migrations are typically dominated by **worker spawn** (tens of milliseconds), not PIR walks (tens of microseconds).

## Guidance

- Prefer long-lived workers in future versions if you need lower latency (not in 0.1)
- Keep captured graphs small — migrate **data**, not whole heaps
- Use `--pir-input` offline fixtures when benchmarking pure analyze/convert
- Release builds (`cargo build --release`) matter for CLI overhead

## Benchmarks directory

See [`benchmarks/README.md`](https://github.com/parallax-runtime/parallax/blob/main/benchmarks/README.md). Criterion crates can be added later; the supported user-facing tool is `plx bench`.
