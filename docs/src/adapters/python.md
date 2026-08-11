# Python adapter

Crate: `parallax-adapter-python`  
Worker: `adapters/python/worker.py`

## Host discovery

Order:

1. `python`
2. `python3`
3. `py`
4. `%LOCALAPPDATA%\Programs\Python\*\python.exe`
5. `%ProgramFiles%\Python\*\python.exe` (and x86)

Candidates that fail `python -c "import sys; print(...)"` or look like the Windows Store stub are skipped.

## Execution model

- Subprocess: `python worker.py`
- Guest code runs via `compile` + `exec` into a dedicated globals dict
- Stdout/stderr of the **guest** are captured separately from the NDJSON control channel
- Named bindings are encoded with the PIR tagged JSON shapes

## Supported value subset (encode)

| Python | PIR |
|---|---|
| `None` | `null` |
| `bool` | `bool` |
| `int` | `int` (decimal string) |
| `float` | `float` |
| `str` | `string` |
| `bytes` | `bytes` |
| `list` | `list` |
| `tuple` | `tuple` |
| `set` | `set` |
| `dict` | `map` |
| callables | `function` |
| other | `unsupported` |

## Restore

PIR → Python values for the supported subset. `bigint` becomes `int`. Functions / unsupported nodes raise `RESTORE_FAILURE`.

## Limitations

- No true local-frame capture beyond post-exec globals
- No continuation / async migration
- Guest `print` is captured; it does not break the protocol channel
