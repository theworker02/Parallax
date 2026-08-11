# Target-language adapters

Targets consume `PUIR` + `ProjectGraph` + `MigrationPlan` (+ style profile) and own:

- syntax emission
- module / file layout
- error & async conventions
- package manifests
- preferred formatter

| Language | Maturity |
|----------|----------|
| Rust | stable (Tier-1 packs) |
| Go | beta |
| Python, TypeScript | experimental |
| Java, Kotlin, C#, Ruby, Swift, Dart | scaffold |

Style profiles (idiomatic / minimal / …) via `plx migrate --target-style` and future `--style`.
