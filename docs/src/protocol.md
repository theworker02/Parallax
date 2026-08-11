# Worker protocol

Versioned NDJSON over stdin/stdout. Implemented in `parallax-protocol` (`PROTOCOL_VERSION = 1`).

## Envelope

Every line is one JSON object:

```json
{
  "v": 1,
  "id": "<uuid>",
  "op": "execute",
  "ok": true,
  "payload": { },
  "error": null
}
```

| Field | Role |
|---|---|
| `v` | Protocol version — mismatch → `ProtocolViolation` |
| `id` | Correlation id (request/response) |
| `op` | Operation name |
| `ok` | Present on responses |
| `payload` | Op-specific JSON |
| `error` | `{ code, message, diagnostic? }` on failure |

## Operations

| `op` | Direction | Purpose |
|---|---|---|
| `hello` | req/resp | Negotiate version; report host/adapter versions |
| `execute` | req/resp | Run source; optional capture list → PIR bindings |
| `restore` | req/resp | Materialize PIR bindings in a fresh context |
| `ping` | req/resp | Liveness |
| `shutdown` | req/resp | Worker exits |

### Execute request (abbrev.)

```json
{
  "source": "state = {'a': 1}",
  "filename": "demo.py",
  "capture": ["state"],
  "limits": { "timeout": 30000, "max_output_bytes": 1048576 }
}
```

`limits.timeout` is milliseconds (serde of `ExecutionLimits`).

### Execute response (abbrev.)

```json
{
  "stdout": "",
  "stderr": "",
  "duration_us": 1234,
  "bindings": { "state": { "t": "map", "entries": [] } },
  "exception": null,
  "success": true
}
```

## Worker locations

Workers are embedded in the adapter crates and materialized under the system temp directory at runtime:

| Runtime | Embedded source | Temp file |
|---|---|---|
| Python | `adapters/python/worker.py` | `%TEMP%/parallax-workers/python_worker.py` |
| JavaScript | `adapters/js/worker.js` | `%TEMP%/parallax-workers/js_worker.js` |

## Timeouts

The core wraps each request in `tokio::time::timeout`. On expiry the worker process is killed and the caller receives `ExecutionTimeout`.
