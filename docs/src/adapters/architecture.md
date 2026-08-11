# Atlas adapter architecture

Parallax Atlas makes language, framework, build, test, database, and deployment support a **modular adapter problem** — not a core rewrite.

```text
Parallax Core / Transmute / Mirror
│
├── parallax-adapter-sdk   formal contracts
└── parallax-atlas         registry, detection, stack planning
        └── built-in adapters (languages, frameworks, …)
```

## Principles

1. **Core orchestrates** — migration planning consumes normalized IR (`PUIR`, `ProjectGraph`) produced or planned via adapters.
2. **Capabilities are explicit** — never assume an adapter supports a construct because it claims a language.
3. **Composition over hardcoding** — TypeScript + NestJS + Prisma + Jest + Docker stack as cooperating adapters with ownership scopes.
4. **Honest maturity** — `stable` / `beta` / `experimental` / `parse_only` / `target_only` / `scaffold`.
5. **Conflicts are visible** — when two adapters of the same kind match, Atlas selects by priority and reports the resolution.

## Crates

| Crate | Role |
|-------|------|
| `parallax-adapter-sdk` | `ParallaxAdapter` trait, manifests, capabilities, detection types |
| `parallax-atlas` | `AdapterRegistry`, built-ins, `analyze_stack`, compatibility, lockfile |
| `parallax-connectors` | Language *identity* catalog (roles, host tools) — complementary to Atlas |
| `parallax-transmute` | Actual TS→Rust (etc.) migration execution |

## Detection → plan

```text
DISCOVER files/manifests
  → CLASSIFY project kind
  → DETECT adapters
  → RESOLVE conflicts (priority)
  → SUGGEST target stack (--to)
  → ESTIMATE coverage (from maturity + pair tier)
```

CLI: `plx analyze . --to rust`

## Lockfile

`parallax.lock` records adapter ids/versions for reproducible migrations (`plx analyze . --write-lock`).

## What Atlas does *not* claim yet

Scaffold adapters detect ecosystems (Java, Spring, Rails, …) but do not fully migrate them. Tier-1 execution remains TypeScript/JavaScript → Rust (Express→Axum pack) with expanding mappings.
