# PIR — Parallax Intermediate Representation

**Schema version:** `1` (`PIR_SCHEMA_VERSION`)

PIR is a tagged JSON value graph used for capture, snapshots, and migration. It is language-neutral and intentionally boring: dictionaries of typed nodes, not bytecode.

## Document shape

```json
{
  "schema": 1,
  "bindings": {
    "state": { "t": "map", "entries": [ /* ... */ ] }
  },
  "objects": {},
  "roots": [],
  "metadata": {}
}
```

- **`bindings`** — primary migration surface (name → value)
- **`objects` / `roots`** — reserved for richer heap graphs (`ref` targets)
- **`metadata`** — free-form; migration fills `migrated_from` / `migrated_to`

## Value tags

| `t` | Payload | Notes |
|---|---|---|
| `null` | — | `None` / `null` / `undefined` |
| `bool` | `v: bool` | |
| `int` | `v: { "decimal": "…" }` | Arbitrary precision decimal text |
| `float` | `v: number` | IEEE-754 binary64 |
| `string` | `v: string` | UTF-8 |
| `bytes` | `v: base64` | |
| `list` | `v: [...]` | Arrays |
| `tuple` | `v: [...]` | Becomes list when targeting JS |
| `set` | `v: [...]` | Becomes list when targeting JS |
| `map` | `entries: [{key,value}]` | Ordered; string keys preferred |
| `bigint` | `v: decimal string` | First-class in JS restore |
| `function` | `name`, `descriptor` | Not migratable |
| `ref` | `id` | Object-graph pointer |
| `unsupported` | `reason`, `repr`, `type_name?` | Explicit failure node |

### Example — demo `state`

```json
{
  "t": "map",
  "entries": [
    { "key": { "t": "string", "v": "username" }, "value": { "t": "string", "v": "Ada" } },
    { "key": { "t": "string", "v": "score" }, "value": { "t": "int", "v": { "decimal": "42" } } },
    {
      "key": { "t": "string", "v": "projects" },
      "value": {
        "t": "list",
        "v": [
          { "t": "string", "v": "compiler" },
          { "t": "string", "v": "runtime" },
          { "t": "string", "v": "vm" }
        ]
      }
    }
  ]
}
```

## Validation

`PirDocument::validate` rejects unknown/future schema versions and dangling roots. Snapshots additionally hash a canonical payload and reject tampering.

## Offline PIR input

```bash
plx migrate path/to/doc.json --to javascript --pir-input
```

Skips live capture; useful for fixtures and fuzz corpora.
