#!/usr/bin/env python3
"""Parallax Python runtime worker — NDJSON protocol on stdin/stdout."""

from __future__ import annotations

import base64
import json
import math
import re
import sys
import time
import traceback
import uuid
from typing import Any, Optional

PROTOCOL_VERSION = 1
ADAPTER_VERSION = "0.1.0"
UES_FORMAT_VERSION = 1

_IDENT_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")
_CHECKPOINT_LINE_RE = re.compile(
    r"^(\s*)(?:parallax\s*\.\s*checkpoint\s*\(|@parallax\.safepoint)"
)


def encode_value(obj: Any) -> Any:
    if obj is None:
        return {"t": "null"}
    if isinstance(obj, bool):
        return {"t": "bool", "v": obj}
    if isinstance(obj, int) and not isinstance(obj, bool):
        return {"t": "int", "v": {"decimal": str(obj)}}
    if isinstance(obj, float):
        if math.isnan(obj):
            return {"t": "float", "v": "NaN"}
        if math.isinf(obj):
            return {"t": "float", "v": "Infinity" if obj > 0 else "-Infinity"}
        return {"t": "float", "v": obj}
    if isinstance(obj, str):
        return {"t": "string", "v": obj}
    if isinstance(obj, bytes):
        return {"t": "bytes", "v": base64.b64encode(obj).decode("ascii")}
    if isinstance(obj, tuple):
        return {"t": "tuple", "v": [encode_value(x) for x in obj]}
    if isinstance(obj, set):
        items = sorted(list(obj), key=lambda x: repr(x))
        return {"t": "set", "v": [encode_value(x) for x in items]}
    if isinstance(obj, list):
        return {"t": "list", "v": [encode_value(x) for x in obj]}
    if isinstance(obj, dict):
        entries = []
        for k, v in obj.items():
            entries.append({"key": encode_value(k), "value": encode_value(v)})
        return {"t": "map", "entries": entries}
    if callable(obj):
        return {
            "t": "function",
            "name": getattr(obj, "__name__", None),
            "descriptor": repr(obj),
        }
    return {
        "t": "unsupported",
        "reason": f"unsupported Python type: {type(obj).__name__}",
        "repr": repr(obj),
        "type_name": type(obj).__name__,
    }


def decode_value(node: Any) -> Any:
    if not isinstance(node, dict) or "t" not in node:
        raise ValueError(f"invalid PIR node: {node!r}")
    t = node["t"]
    if t == "null":
        return None
    if t == "bool":
        return bool(node["v"])
    if t == "int":
        v = node["v"]
        if isinstance(v, dict):
            return int(v["decimal"])
        return int(v)
    if t == "float":
        v = node["v"]
        if v == "NaN":
            return float("nan")
        if v in ("Infinity", "+Infinity"):
            return float("inf")
        if v == "-Infinity":
            return float("-inf")
        return float(v)
    if t == "string":
        return node["v"]
    if t == "bytes":
        return base64.b64decode(node["v"])
    if t == "list":
        return [decode_value(x) for x in node.get("v", [])]
    if t == "tuple":
        return tuple(decode_value(x) for x in node.get("v", []))
    if t == "set":
        return set(decode_value(x) for x in node.get("v", []))
    if t == "map":
        out = {}
        for e in node.get("entries", []):
            key = decode_value(e["key"])
            out[key] = decode_value(e["value"])
        return out
    if t in ("bigint", "big_int"):
        return int(node["v"])
    if t == "function":
        raise ValueError("cannot restore function")
    if t == "unsupported":
        raise ValueError(f"cannot restore unsupported: {node.get('reason')}")
    if t == "ref":
        raise ValueError("ref restore not implemented in Python worker")
    raise ValueError(f"unknown PIR type: {t}")


class CheckpointPause(BaseException):
    """Cooperative Continuum pause — not a guest failure."""

    def __init__(self, label: str, locals_dict: dict, globals_dict: dict):
        super().__init__(label)
        self.label = label
        self.locals_dict = locals_dict
        self.globals_dict = globals_dict


def split_at_checkpoint(source: str) -> tuple[str, Optional[str], Optional[str]]:
    """Split source into (pre_including_checkpoint_call, post, label_hint).

    Resume runs *only* the post region with restored bindings — never restarts
    the pre-checkpoint region from the beginning.
    """
    lines = source.splitlines(keepends=True)
    for i, line in enumerate(lines):
        if _CHECKPOINT_LINE_RE.search(line):
            pre = "".join(lines[: i + 1])
            post = "".join(lines[i + 1 :])
            label = "checkpoint"
            m = re.search(r"checkpoint\s*\(\s*['\"]([^'\"]+)['\"]", line)
            if m:
                label = m.group(1)
            return pre, post if post.strip() else "", label
    return source, None, None


def build_parallax_module(resume_holder: dict):
    """Injected guest module providing parallax.checkpoint / safepoint."""

    class ParallaxMod:
        @staticmethod
        def checkpoint(label: str = "checkpoint", locals_=None):
            frame = sys._getframe(1)
            locs = dict(locals_) if locals_ is not None else {
                k: v
                for k, v in frame.f_locals.items()
                if not k.startswith("__") and k != "parallax"
            }
            globs = {
                k: v
                for k, v in frame.f_globals.items()
                if not k.startswith("__") and k not in ("parallax",)
            }
            resume_holder["label"] = label
            raise CheckpointPause(label, locs, globs)

        @staticmethod
        def safepoint(label: str = "safepoint"):
            ParallaxMod.checkpoint(label)

    return ParallaxMod()


def build_ues(
    label: str,
    filename: str,
    locals_dict: dict,
    globals_dict: dict,
    resume_source: Optional[str],
) -> dict:
    skip = {"parallax"}
    locals_pir = {
        k: encode_value(v) for k, v in locals_dict.items() if k not in skip and _IDENT_RE.match(k)
    }
    globals_pir = {
        k: encode_value(v)
        for k, v in globals_dict.items()
        if k not in skip and _IDENT_RE.match(k)
    }
    # Prefer locals; merge globals for heap bindings root.
    bindings = dict(globals_pir)
    bindings.update(locals_pir)
    frame_id = str(uuid.uuid4())
    return {
        "format_version": UES_FORMAT_VERSION,
        "execution_id": str(uuid.uuid4()),
        "source_runtime": "python",
        "source_program": filename,
        "control_state": {
            "safepoint_label": label,
            "safepoint_kind": "explicit_checkpoint",
            "instruction_position": f"safepoint:{label}",
            "suspended": True,
        },
        "call_stack": [
            {
                "frame_id": frame_id,
                "runtime": "python",
                "function": "__parallax_main",
                "module": filename,
                "source_location": None,
                "instruction_position": f"safepoint:{label}",
                "arguments": {},
                "locals": locals_pir,
                "temporaries": {},
                "return_target": None,
                "exception_target": None,
                "locals_root": None,
                "runtime_metadata": {"safepoint_kind": "explicit_checkpoint"},
            }
        ],
        "heap": {"bindings": bindings},
        "globals": globals_pir,
        "modules": [{"name": filename, "runtime": "python", "fingerprint": None}],
        "exception_state": None,
        "async_state": {"status": "none"},
        "capability_state": {
            "granted": ["explicit_checkpoint"],
            "notes": ["Arbitrary live stack migration is NOT claimed"],
        },
        "external_resources": [],
        "deterministic_context": {
            "engine_status": "unsupported",
            "unsupported_reason": "Deterministic replay engine is not implemented; journal schema only",
            "extensions": {},
        },
        "migration_metadata": {
            "capture_mode": "explicit_checkpoint",
            "continuum_status": "experimental",
            "notes": [
                "Captured at explicit parallax.checkpoint safepoint",
                "Same-runtime resume executes post-checkpoint source only",
            ],
            "extra": {},
        },
        # Host fills a validated PCIR checkpoint stub; worker omits wire-fragile ops.
        "pcir": None,
        "resume_source": resume_source,
        "extensions": {},
    }


def safepoint_report(label: str) -> dict:
    return {
        "kind": "explicit_checkpoint",
        "label": label,
        "runtime": "python",
        "can_capture": "YES",
        "can_snapshot": "YES",
        "can_replay": "UNSUPPORTED",
        "can_migrate": "PARTIAL",
        "targets": ["python"],
        "capability_levels": {
            "capture": "experimental",
            "snapshot": "experimental",
            "replay": "no",
            "same_runtime_resume": "experimental",
            "cross_runtime_resume": "no",
        },
        "semantic_loss": [
            {
                "code": "no_live_stack",
                "message": "Only explicit checkpoint regions are captured; arbitrary frames are not",
            }
        ],
        "status": "EXPERIMENTAL",
        "notes": [
            "Hit via parallax.checkpoint() / @parallax.safepoint",
            "Cross-runtime continuation resume is Unsupported",
        ],
    }


def respond(req_id, op: str, ok: bool, payload=None, error=None):
    msg = {"v": PROTOCOL_VERSION, "id": req_id, "op": op, "ok": ok}
    if payload is not None:
        msg["payload"] = payload
    if error is not None:
        msg["error"] = error
    sys.stdout.write(json.dumps(msg, allow_nan=False) + "\n")
    sys.stdout.flush()


def handle_hello(req_id, payload):
    peer = payload.get("protocol_version")
    if peer is not None and int(peer) != PROTOCOL_VERSION:
        respond(
            req_id,
            "hello",
            False,
            error={
                "code": "PROTOCOL_VIOLATION",
                "message": f"protocol version mismatch: got {peer}, expected {PROTOCOL_VERSION}",
            },
        )
        return
    respond(
        req_id,
        "hello",
        True,
        {
            "protocol_version": PROTOCOL_VERSION,
            "runtime": "python",
            "host_version": f"{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}",
            "adapter_version": ADAPTER_VERSION,
        },
    )


def handle_execute(req_id, payload):
    source = payload.get("source", "")
    filename = payload.get("filename", "<parallax>")
    capture = payload.get("capture") or []
    continuum = bool(payload.get("continuum"))
    for name in capture:
        if not isinstance(name, str) or not _IDENT_RE.match(name):
            respond(
                req_id,
                "execute",
                False,
                error={
                    "code": "PROTOCOL_VIOLATION",
                    "message": f"invalid capture binding name: {name!r}",
                },
            )
            return
    t0 = time.perf_counter()
    resume_holder: dict = {}
    g: dict[str, Any] = {"__name__": "__parallax_guest__"}
    if continuum:
        g["parallax"] = build_parallax_module(resume_holder)
    stdout_buf: list[str] = []
    stderr_buf: list[str] = []

    class Capture:
        def __init__(self, buf):
            self.buf = buf

        def write(self, s):
            self.buf.append(s)
            return len(s)

        def flush(self):
            pass

    old_out, old_err = sys.stdout, sys.stderr
    success = True
    suspended = False
    exception = None
    ues = None
    sp_report = None
    run_source = source
    resume_source = None
    if continuum:
        pre, post, _hint = split_at_checkpoint(source)
        if post is not None:
            run_source = pre
            resume_source = post

    try:
        sys.stdout = Capture(stdout_buf)  # type: ignore
        sys.stderr = Capture(stderr_buf)  # type: ignore
        code = compile(run_source, filename, "exec")
        exec(code, g, g)
    except CheckpointPause as cp:
        suspended = True
        success = True
        label = cp.label
        ues = build_ues(label, filename, cp.locals_dict, cp.globals_dict, resume_source)
        sp_report = safepoint_report(label)
    except BaseException as e:
        success = False
        exception = {
            "type_name": type(e).__name__,
            "message": str(e),
            "stack": traceback.format_exc(),
        }
    finally:
        sys.stdout = old_out
        sys.stderr = old_err

    bindings = {}
    if success and not suspended:
        for name in capture:
            if name in g:
                bindings[name] = encode_value(g[name])
            else:
                success = False
                exception = {
                    "type_name": "NameError",
                    "message": f"binding '{name}' not defined after execution",
                    "stack": None,
                }
                break
    elif suspended and ues is not None:
        bindings = ues.get("heap", {}).get("bindings") or {}

    duration_us = int((time.perf_counter() - t0) * 1_000_000)
    payload_out = {
        "stdout": "".join(stdout_buf),
        "stderr": "".join(stderr_buf),
        "duration_us": duration_us,
        "bindings": bindings,
        "exception": exception,
        "success": success,
        "suspended": suspended,
    }
    if ues is not None:
        payload_out["ues"] = ues
    if sp_report is not None:
        payload_out["safepoint"] = sp_report
    respond(req_id, "execute", True, payload_out)


def handle_resume_checkpoint(req_id, payload):
    """Resume post-checkpoint source with restored bindings — not a full restart."""
    t0 = time.perf_counter()
    ues = payload.get("ues") or {}
    if ues.get("source_runtime") not in (None, "python"):
        respond(
            req_id,
            "resume_checkpoint",
            False,
            error={
                "code": "CAPABILITY_VIOLATION",
                "message": "cross-runtime continuation resume is Unsupported",
                "diagnostic": f"ues.source_runtime={ues.get('source_runtime')}",
            },
        )
        return
    fmt = ues.get("format_version")
    if fmt is not None and int(fmt) != UES_FORMAT_VERSION:
        respond(
            req_id,
            "resume_checkpoint",
            False,
            error={
                "code": "INVALID_SNAPSHOT",
                "message": f"unsupported UES format version {fmt}",
            },
        )
        return
    resume_source = ues.get("resume_source")
    if resume_source is None:
        respond(
            req_id,
            "resume_checkpoint",
            False,
            error={
                "code": "UNSUPPORTED_VALUE",
                "message": "UES has no resume_source; cannot resume without restarting",
            },
        )
        return
    if ues.get("control_state", {}).get("safepoint_kind") != "explicit_checkpoint":
        respond(
            req_id,
            "resume_checkpoint",
            False,
            error={
                "code": "UNSUPPORTED_VALUE",
                "message": "only explicit_checkpoint UES can be resumed",
            },
        )
        return

    bindings_in = (ues.get("heap") or {}).get("bindings") or {}
    # Prefer frame locals when present.
    stack = ues.get("call_stack") or []
    if stack and isinstance(stack[0], dict) and stack[0].get("locals"):
        merged = dict(bindings_in)
        merged.update(stack[0]["locals"])
        bindings_in = merged

    g: dict[str, Any] = {"__name__": "__parallax_guest__"}
    warnings = ["EXPERIMENTAL: same-runtime checkpoint resume"]
    try:
        for name, node in bindings_in.items():
            if not isinstance(name, str) or not _IDENT_RE.match(name):
                continue
            try:
                g[name] = decode_value(node)
            except Exception as e:
                warnings.append(f"skip binding {name}: {e}")
    except Exception as e:
        respond(
            req_id,
            "resume_checkpoint",
            False,
            error={
                "code": "RESTORE_FAILURE",
                "message": str(e),
                "diagnostic": traceback.format_exc(),
            },
        )
        return

    stdout_buf: list[str] = []
    stderr_buf: list[str] = []

    class Capture:
        def __init__(self, buf):
            self.buf = buf

        def write(self, s):
            self.buf.append(s)
            return len(s)

        def flush(self):
            pass

    old_out, old_err = sys.stdout, sys.stderr
    success = True
    exception = None
    try:
        sys.stdout = Capture(stdout_buf)  # type: ignore
        sys.stderr = Capture(stderr_buf)  # type: ignore
        if resume_source.strip():
            code = compile(resume_source, ues.get("source_program") or "<resume>", "exec")
            exec(code, g, g)
    except BaseException as e:
        success = False
        exception = {
            "type_name": type(e).__name__,
            "message": str(e),
            "stack": traceback.format_exc(),
        }
    finally:
        sys.stdout = old_out
        sys.stderr = old_err

    out_bindings = {
        k: encode_value(v)
        for k, v in g.items()
        if _IDENT_RE.match(k) and not k.startswith("__")
    }
    duration_us = int((time.perf_counter() - t0) * 1_000_000)
    respond(
        req_id,
        "resume_checkpoint",
        True,
        {
            "success": success,
            "stdout": "".join(stdout_buf),
            "stderr": "".join(stderr_buf),
            "duration_us": duration_us,
            "bindings": out_bindings,
            "exception": exception,
            "warnings": warnings,
        },
    )


def handle_restore(req_id, payload):
    t0 = time.perf_counter()
    bindings_in = payload.get("bindings") or {}
    restored = {}
    warnings = []
    try:
        g = {}
        for name, node in bindings_in.items():
            g[name] = decode_value(node)
            restored[name] = type(g[name]).__name__
        out_bindings = {k: encode_value(v) for k, v in g.items()}
        duration_us = int((time.perf_counter() - t0) * 1_000_000)
        respond(
            req_id,
            "restore",
            True,
            {
                "success": True,
                "warnings": warnings,
                "restored": restored,
                "duration_us": duration_us,
                "bindings": out_bindings,
            },
        )
    except Exception as e:
        respond(
            req_id,
            "restore",
            False,
            error={
                "code": "RESTORE_FAILURE",
                "message": str(e),
                "diagnostic": traceback.format_exc(),
            },
        )


def handle_shutdown(req_id, _payload):
    respond(req_id, "shutdown", True, {})
    sys.exit(0)


HANDLERS = {
    "hello": handle_hello,
    "execute": handle_execute,
    "restore": handle_restore,
    "resume_checkpoint": handle_resume_checkpoint,
    "shutdown": handle_shutdown,
    "ping": lambda req_id, _p: respond(req_id, "ping", True, {"pong": True}),
}


def main():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            msg = json.loads(line)
        except json.JSONDecodeError as e:
            sys.stderr.write(f"protocol error: {e}\n")
            continue
        req_id = msg.get("id")
        op = msg.get("op")
        payload = msg.get("payload") or {}
        peer_v = msg.get("v")
        if peer_v is not None and int(peer_v) != PROTOCOL_VERSION:
            respond(
                req_id,
                op or "unknown",
                False,
                error={
                    "code": "PROTOCOL_VIOLATION",
                    "message": f"protocol version mismatch: got {peer_v}, expected {PROTOCOL_VERSION}",
                },
            )
            continue
        handler = HANDLERS.get(op)
        if handler is None:
            respond(
                req_id,
                op or "unknown",
                False,
                error={"code": "PROTOCOL_VIOLATION", "message": f"unknown op: {op}"},
            )
            continue
        try:
            handler(req_id, payload)
        except Exception as e:
            respond(
                req_id,
                op,
                False,
                error={
                    "code": "INTERNAL",
                    "message": str(e),
                    "diagnostic": traceback.format_exc(),
                },
            )


if __name__ == "__main__":
    main()
