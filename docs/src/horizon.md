# Event Horizon

Event Horizon is Parallax's **semantic reconstruction** layer for migrations that look "impossible" on paper — dynamic Python, open dispatch, eval, native extensions, and other constructs with no direct target equivalent.

Philosophy: **no direct equivalent ≠ migration impossible**. Preserve behavior via native lowering, specialized capsules, polyglot islands, or (eventually) behavioral synthesis — never silent semantic drift.

## Crate layout (consolidated)

Phase X originally scaffolded ~20 small crates; they are merged into **one** workspace member:

| Crate | Role |
|-------|------|
| `parallax-horizon` | Full Event Horizon stack — PVABI (`pvabi/`), semantics, behavior, IR, VCS stubs, and orchestration (`analyze_impossible`, debt, dissolve, detach, etc.) |

Inside `parallax-horizon`, modules are grouped as `pvabi/`, `semantics/`, `behavior/`, `ir/`, and `vcs/`. Public types are re-exported at the crate root (e.g. `ProjectObserver`, `PreservationPolicy`, `SemanticPatch`) and also available via submodule paths (e.g. `parallax_horizon::pvabi::PvValue`, `parallax_horizon::behavior::ProjectObserver`, `parallax_horizon::vcs::SemanticPatch`).

## CLI commands

Top-level `plx` commands (Windows stack size is raised in `parallax-cli/build.rs`):

| Command | Purpose |
|---------|---------|
| `plx observe [path]` | Static semantic inspection (languages, dynamic signals, effects) |
| `plx impossible [path] --to rust` | Hard barriers + proposed preservation strategies |
| `plx dissolve [path]` | One-step polyglot island shrink (scaffold) |
| `plx debt [path]` | Compatibility debt / target purity score |
| `plx detach [path]` | Whether source runtime can be dropped (threshold-based) |
| `plx reconstruct` | Behavioral reconstruction status (scaffold) |
| `plx optimize-migration [path]` | Propose native replacements for capsules/islands |
| `plx explain-barrier --id N [path]` | Explain a barrier from `plx impossible` |
| `plx blame <file:line>` | Semantic blame (scaffold — needs `.plxmap.json`) |
| `plx cherry-pick <commit>` | Semantic cherry-pick (scaffold) |
| `plx patch --example` | Print example `.plxp` semantic patch |

Use `--json` on any command for machine-readable output.

## Example

```bash
plx observe examples/hostile-dynamic
plx impossible examples/hostile-dynamic --to rust
plx debt examples/hostile-dynamic --to rust
```

The [hostile-dynamic](../examples/hostile-dynamic/) fixture includes `getattr`, decorators, and `asyncio` signals.

## Honest limits

- **Observatory** uses static heuristics, not dynamic tracing.
- **Behavior synthesis / CEGIS / reconstruct** are scaffolds — no end-to-end behavioral equivalence yet.
- **Semantic git** (`blame`, `cherry-pick`) requires `.plxmap.json` + git linkage (not wired).
- **Debt / dissolve / detach** scores are estimates from preservation decisions — verify with tests before production detach.
- Transmute / Mirror / weather-api paths are unchanged; Horizon is additive analysis tooling.

See also: [Transmute](./transmute.md), [Mirror](./mirror.md), [Limitations](./limitations.md).
