# Benchmarks

Use the CLI for measured end-to-end timings:

```bash
cargo run -p parallax-cli --bin plx -- bench --iterations 20 --json
```

Unit-level Criterion benches can be added under individual crates later (`[[bench]]`).
