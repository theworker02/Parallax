# Hostile dynamic fixture

Small Python project used to exercise Event Horizon (`plx observe`, `plx impossible`, `plx debt`).

Signals intentionally included:

- `getattr` / dynamic attribute access
- `@dataclass` and custom decorators
- `asyncio` async functions
- FastAPI in `requirements.txt` (framework detection)

## Related demos

| Example | Command |
|---|---|
| Stack detection (Nest + Prisma) | `plx analyze examples/stacks/nest-prisma --to rust` |
| Stack detection (FastAPI + SQLAlchemy) | `plx analyze examples/stacks/fastapi-sqlalchemy --to rust` |
| Transmute reference | `plx migrate examples/weather-api --to rust -o examples/weather-api-rust` |
| Horizon debt | `plx debt examples/hostile-dynamic --to rust` |

```bash
plx observe examples/hostile-dynamic
plx impossible examples/hostile-dynamic --to rust
plx debt examples/hostile-dynamic --to rust
```
