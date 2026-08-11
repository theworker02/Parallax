# Migration demo

The canonical demo migrates a small Python object into JavaScript (and back).

## Source (`examples/demo.py`)

```python
state = {
    "username": "Ada",
    "score": 42,
    "projects": ["compiler", "runtime", "vm"],
}
print("python state ready:", state)
```

## Python → JavaScript

```bash
plx migrate examples/demo.py --to javascript -o examples/demo.migrated.js
```

What happens:

1. **Capture** — Python worker executes the file and encodes `state` as PIR
2. **Analyze** — semantic-loss pass for JavaScript
3. **Convert** — PIR rewritten under the default conversion policy
4. **Restore** — JavaScript worker materializes the bindings
5. **Emit** — optional JS source preview written to `-o`

You should see measured timings similar to:

```text
Migration python → javascript (OK)
timings:
  capture: … µs
  analyze: … µs
  convert: … µs
  restore: … µs
  total:   … µs

migrated bindings:
  state: map{3}
```

Numbers are **wall-clock measurements**, not placeholders.

Emitted preview (shape):

```javascript
const state = { "username": "Ada", "score": 42, "projects": ["compiler", "runtime", "vm"] };
```

## JavaScript → Python

```bash
plx migrate examples/demo.js --to python -o examples/demo.migrated.py
```

## Integer precision policy

`examples/demo_bigint.py` uses a value outside the JS safe integer range (`9007199254740993`).

| Flags | Result |
|---|---|
| *(default)* | Convert to **BigInt** (`prefer_bigint=true`) — migration **OK**, finding `SAFE` |
| `--no-prefer-bigint` | **Rejected** as `LOSSY` / `MigrationRejected` |
| `--no-prefer-bigint --allow-lossy` | Coerce to JS `Number` (opt-in precision loss) |

```bash
plx migrate examples/demo_bigint.py --to javascript --no-prefer-bigint
# → fails with structured MigrationRejected

plx migrate examples/demo_bigint.py --to javascript -o examples/demo_bigint.migrated.js
# → score becomes 9007199254740993n in the preview
```

## Snapshot along the way

```bash
plx migrate examples/demo.py --to javascript \
  --snapshot /tmp/demo.migrated.plx \
  -o /tmp/demo.migrated.js

plx inspect /tmp/demo.migrated.plx
```

## Machine-readable report

```bash
plx migrate examples/demo.py --to javascript --json
```

Includes the migration report (findings + timings) and the migrated PIR bindings object.
