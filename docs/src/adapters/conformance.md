# Adapter conformance

Levels:

### Bronze

- Parses / detects project
- Emits valid IR or structured detection
- Basic fixtures pass

### Silver

- Target compiles for supported packs
- Tests migrate where claimed
- Dependency mapping works

### Gold

- Behavioral verification
- Edge-case suite
- Incremental sync where applicable
- High fixture coverage

`plx adapters health` exposes a heuristic score from maturity + conformance medals. Full fixture-driven scoring expands with `parallax-adapter-testkit`.
