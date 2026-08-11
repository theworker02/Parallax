# JavaScript adapter

Crate: `parallax-adapter-js`  
Worker: `adapters/js/worker.js`

## Host discovery

Order:

1. `node`
2. `nodejs`
3. `%ProgramFiles%\nodejs\node.exe`

## Execution model

- Subprocess: `node worker.js`
- Guest code runs in `vm.Script` / `vm.createContext`
- Capture works for top-level `let` / `const` / `var` by appending a final expression that reads names from script scope
- `console.log` / `error` / `warn` are redirected into captured stdout/stderr buffers

## Supported value subset (encode)

| JavaScript | PIR |
|---|---|
| `null` / `undefined` | `null` |
| `boolean` | `bool` |
| safe integer `number` | `int` |
| other `number` | `float` |
| `bigint` | `bigint` |
| `string` | `string` |
| `Buffer` / `Uint8Array` | `bytes` |
| `Array` | `list` |
| `Set` | `set` |
| plain object / `Map` | `map` |
| `function` | `function` |
| other | `unsupported` |

## Restore

- PIR `int` within the safe integer range → `number`
- Larger ints / `bigint` → JS `BigInt`
- `map` with string keys → plain object
- `list` / `tuple` / `set` → `Array` (set semantics not reified as `Set` today)

## Limitations

- No DOM / browser engine — **Node.js only** in 0.1
- No async migration
- Module `import` / ESM loader hooks are not provided inside the vm context
