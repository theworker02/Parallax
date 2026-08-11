# Mirror — continuous cross-language sync

**Mirror** keeps a migrated target project synchronized with an evolving source project. After Transmute produces a first target, link the pair and sync incrementally instead of remigrating the whole repository.

```text
TypeScript changes
        ↓
semantic diff vs baseline PUIR
        ↓
regenerate affected regions
        ↓
build + differential tests
        ↓
Rust (or other target) updated
```

## First supported path

| Source | Target | Tier |
|--------|--------|------|
| TypeScript / JavaScript | Rust | **Tier 1** (weather-api demo) |
| Python | Rust | Tier 2 (planning / partial) |
| Rust → TypeScript | — | Experimental (reverse sync gated) |

Tiers reflect implemented conformance, not marketing claims.

## Workflow

```bash
# One-time: migrate, then link
plx migrate ./examples/weather-api --to rust -o ./examples/weather-api-rust --require-build --require-tests
plx link ./examples/weather-api ./examples/weather-api-rust

# Ongoing
# edit TypeScript…
plx sync
plx sync --check    # CI / no writes
plx status          # drift summary
plx ci              # sync --check + target verify
```

Link metadata lives under the **target**:

```text
.parallax-link/
├── link.json
├── source-index.bin
├── semantic-map.bin
├── dependency-map.json
├── manual-regions.json
├── ownership.json
├── baselines/
└── history/
```

Default policy: `source-authoritative`.

## Commands

| Command | Purpose |
|---------|---------|
| `plx link <src> <tgt>` | Create Mirror link |
| `plx sync` | Incremental translate + verify |
| `plx sync --check` | Freshness only (fails if stale) |
| `plx sync --reverse` | Target→source when node is ExactYes (else Unsupported) |
| `plx sync --patch` | Preview regenerate without writing |
| `plx status` / `--json` | Drift / machine-readable status |
| `plx ci` | `sync --check` + differential verify |
| `plx history` | Sync history |
| `plx rollback` | Restore last pre-sync target snapshot |
| `plx explain <file:line> -C <tgt>` | Source-map explanation |
| `plx why <file> -C <tgt>` | Why a target file changed |
| `plx verify` | Run differential / property notes |

## What sync does (and does not)

**Does**

- Semantic diff of PUIR (not text-only)
- Stable semantic IDs (`plx:function:…`)
- Regenerate affected modules transactionally (snapshot → apply → build/test → commit or restore)
- Preserve files marked with manual `BEHAVIOR_CHANGE` ownership
- Report testing confidence (migrated suite), never claim formal proof

**Does not (yet / honest limits)**

- Full bidirectional live migration without review gates
- Property-based equivalence as a complete fuzzer (`--property` notes only)
- `plx watch` daemon (not shipped)
- Guaranteed idiom preservation for every manual refactor
- All language pairs as Tier 1

## Manual edits

Ownership metadata is stored in `.parallax-link/ownership.json` and `manual-regions.json` (sidecar preferred over invasive markers). Behavior-changing target edits block overwrite under source-authoritative policy until you choose preserve / overwrite / inspect.

## CI

```yaml
- run: plx sync --check
- run: plx ci
```

Fails when the link is stale, the target fails build/tests, or verification mismatches.

## Related

- [Transmute](transmute.md) — initial project migration
- [CLI](cli.md) — command reference
- [Limitations](limitations.md) — capability honesty
