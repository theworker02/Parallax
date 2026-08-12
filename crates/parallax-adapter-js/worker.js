#!/usr/bin/env node
"use strict";

/**
 * Parallax JavaScript (Node.js) runtime worker — NDJSON on stdin/stdout.
 */

const PROTOCOL_VERSION = 1;
const ADAPTER_VERSION = "0.1.0";
const UES_FORMAT_VERSION = 1;
const crypto = require("crypto");
const readline = require("readline");
const vm = require("vm");

function isIdent(name) {
  return typeof name === "string" && /^[A-Za-z_][A-Za-z0-9_]*$/.test(name);
}

function encodeValue(obj) {
  // Do not collapse JS `undefined` into PIR null — that silently merges distinct values.
  if (obj === undefined) {
    return {
      t: "unsupported",
      reason: "javascript_undefined",
      repr: "undefined",
      type_name: "undefined",
    };
  }
  if (obj === null) {
    return { t: "null" };
  }
  const ty = typeof obj;
  if (ty === "boolean") {
    return { t: "bool", v: obj };
  }
  if (ty === "number") {
    if (Number.isNaN(obj)) {
      return { t: "float", v: "NaN" };
    }
    if (obj === Infinity) {
      return { t: "float", v: "Infinity" };
    }
    if (obj === -Infinity) {
      return { t: "float", v: "-Infinity" };
    }
    if (Number.isInteger(obj) && Number.isSafeInteger(obj)) {
      return { t: "int", v: { decimal: String(obj) } };
    }
    return { t: "float", v: obj };
  }
  if (ty === "bigint") {
    return { t: "bigint", v: obj.toString() };
  }
  if (ty === "string") {
    return { t: "string", v: obj };
  }
  if (ty === "function") {
    return {
      t: "function",
      name: obj.name || null,
      descriptor: Function.prototype.toString.call(obj).slice(0, 200),
    };
  }
  if (Buffer.isBuffer(obj) || obj instanceof Uint8Array) {
    return { t: "bytes", v: Buffer.from(obj).toString("base64") };
  }
  if (Array.isArray(obj)) {
    return { t: "list", v: obj.map(encodeValue) };
  }
  if (obj instanceof Set) {
    return { t: "set", v: Array.from(obj).map(encodeValue) };
  }
  if (obj instanceof Map) {
    const entries = [];
    for (const [k, v] of obj.entries()) {
      entries.push({ key: encodeValue(k), value: encodeValue(v) });
    }
    return { t: "map", entries };
  }
  if (ty === "object") {
    const entries = [];
    for (const [k, v] of Object.entries(obj)) {
      entries.push({ key: encodeValue(k), value: encodeValue(v) });
    }
    return { t: "map", entries };
  }
  return {
    t: "unsupported",
    reason: `unsupported JS type: ${ty}`,
    repr: String(obj),
    type_name: ty,
  };
}

function decodeValue(node) {
  if (!node || typeof node !== "object" || !node.t) {
    throw new Error(`invalid PIR node: ${JSON.stringify(node)}`);
  }
  switch (node.t) {
    case "null":
      return null;
    case "bool":
      return !!node.v;
    case "int": {
      const dec = typeof node.v === "object" ? node.v.decimal : String(node.v);
      const n = Number(dec);
      if (Number.isSafeInteger(n) && String(n) === dec) {
        return n;
      }
      return BigInt(dec);
    }
    case "float": {
      if (node.v === "NaN") return NaN;
      if (node.v === "Infinity" || node.v === "+Infinity") return Infinity;
      if (node.v === "-Infinity") return -Infinity;
      return Number(node.v);
    }
    case "string":
      return node.v;
    case "bytes":
      return new Uint8Array(Buffer.from(node.v, "base64"));
    case "list":
    case "tuple":
      return (node.v || []).map(decodeValue);
    case "set":
      return new Set((node.v || []).map(decodeValue));
    case "map": {
      const out = {};
      for (const e of node.entries || []) {
        const k = decodeValue(e.key);
        out[typeof k === "string" ? k : String(k)] = decodeValue(e.value);
      }
      return out;
    }
    case "bigint":
    case "big_int":
      return BigInt(node.v);
    case "function":
      throw new Error("cannot restore function");
    case "unsupported":
      throw new Error(`cannot restore unsupported: ${node.reason}`);
    case "ref":
      throw new Error("ref restore not implemented in JS worker");
    default:
      throw new Error(`unknown PIR type: ${node.t}`);
  }
}

function respond(reqId, op, ok, payload, error) {
  const msg = { v: PROTOCOL_VERSION, id: reqId, op, ok };
  if (payload !== undefined) msg.payload = payload;
  if (error !== undefined) msg.error = error;
  process.stdout.write(JSON.stringify(msg) + "\n");
}

function splitAtCheckpoint(source) {
  const lines = source.split(/(?<=\n)/);
  const re = /^\s*parallax\s*\.\s*checkpoint\s*\(/;
  for (let i = 0; i < lines.length; i++) {
    if (re.test(lines[i])) {
      const pre = lines.slice(0, i + 1).join("");
      const post = lines.slice(i + 1).join("");
      let label = "checkpoint";
      const m = lines[i].match(/checkpoint\s*\(\s*['"]([^'"]+)['"]/);
      if (m) label = m[1];
      return { pre, post, label };
    }
  }
  return { pre: source, post: null, label: null };
}

function buildUes(label, filename, localsObj, resumeSource) {
  const localsPir = {};
  for (const [k, v] of Object.entries(localsObj || {})) {
    if (isIdent(k) && k !== "parallax" && k !== "console") {
      localsPir[k] = encodeValue(v);
    }
  }
  const frameId = crypto.randomUUID();
  return {
    format_version: UES_FORMAT_VERSION,
    execution_id: crypto.randomUUID(),
    source_runtime: "javascript",
    source_program: filename,
    control_state: {
      safepoint_label: label,
      safepoint_kind: "explicit_checkpoint",
      instruction_position: `safepoint:${label}`,
      suspended: true,
    },
    call_stack: [
      {
        frame_id: frameId,
        runtime: "javascript",
        function: "__parallax_main",
        module: filename,
        source_location: null,
        instruction_position: `safepoint:${label}`,
        arguments: {},
        locals: localsPir,
        temporaries: {},
        return_target: null,
        exception_target: null,
        locals_root: null,
        runtime_metadata: { safepoint_kind: "explicit_checkpoint" },
      },
    ],
    heap: { bindings: { ...localsPir } },
    globals: { ...localsPir },
    modules: [{ name: filename, runtime: "javascript", fingerprint: null }],
    exception_state: null,
    async_state: { status: "none" },
    capability_state: {
      granted: ["explicit_checkpoint"],
      notes: ["Arbitrary live stack migration is NOT claimed"],
    },
    external_resources: [],
    deterministic_context: {
      engine_status: "unsupported",
      unsupported_reason:
        "Deterministic replay engine is not implemented; journal schema only",
      extensions: {},
    },
    migration_metadata: {
      capture_mode: "explicit_checkpoint",
      continuum_status: "experimental",
      notes: [
        "Captured at explicit parallax.checkpoint safepoint",
        "Same-runtime resume executes post-checkpoint source only",
      ],
      extra: {},
    },
    pcir: null,
    resume_source: resumeSource,
    extensions: {},
  };
}

function safepointReport(label) {
  return {
    kind: "explicit_checkpoint",
    label,
    runtime: "javascript",
    can_capture: "YES",
    can_snapshot: "YES",
    can_replay: "UNSUPPORTED",
    can_migrate: "PARTIAL",
    targets: ["javascript"],
    capability_levels: {
      capture: "experimental",
      snapshot: "experimental",
      replay: "no",
      same_runtime_resume: "experimental",
      cross_runtime_resume: "no",
    },
    semantic_loss: [
      {
        code: "no_live_stack",
        message:
          "Only explicit checkpoint regions are captured; arbitrary frames are not",
      },
    ],
    status: "EXPERIMENTAL",
    notes: [
      "Hit via parallax.checkpoint()",
      "Cross-runtime continuation resume is Unsupported",
    ],
  };
}

function handleHello(reqId, payload) {
  const peer = payload && payload.protocol_version;
  if (peer !== undefined && Number(peer) !== PROTOCOL_VERSION) {
    respond(reqId, "hello", false, undefined, {
      code: "PROTOCOL_VIOLATION",
      message: `protocol version mismatch: got ${peer}, expected ${PROTOCOL_VERSION}`,
    });
    return;
  }
  respond(reqId, "hello", true, {
    protocol_version: PROTOCOL_VERSION,
    runtime: "javascript",
    host_version: process.versions.node,
    adapter_version: ADAPTER_VERSION,
  });
}

function handleExecute(reqId, payload) {
  const source = payload.source || "";
  const filename = payload.filename || "<parallax>";
  const capture = payload.capture || [];
  const continuum = !!payload.continuum;
  const t0 = process.hrtime.bigint();

  for (const name of capture) {
    if (!isIdent(name)) {
      respond(reqId, "execute", false, undefined, {
        code: "PROTOCOL_VIOLATION",
        message: `invalid capture binding name: ${name}`,
      });
      return;
    }
  }

  let stdout = "";
  let stderr = "";
  let runSource = source;
  let resumeSource = null;
  if (continuum) {
    const split = splitAtCheckpoint(source);
    if (split.post !== null) {
      runSource = split.pre;
      resumeSource = split.post;
    }
  }

  const sandbox = {
    console: {
      log: (...args) => {
        stdout += args.map(String).join(" ") + "\n";
      },
      error: (...args) => {
        stderr += args.map(String).join(" ") + "\n";
      },
      warn: (...args) => {
        stderr += args.map(String).join(" ") + "\n";
      },
    },
  };
  if (continuum) {
    sandbox.parallax = {
      checkpoint(label, localsObj) {
        const err = new Error("PARALLAX_CHECKPOINT");
        err.name = "CheckpointPause";
        err.parallaxCheckpoint = true;
        err.label = label || "checkpoint";
        err.locals = localsObj && typeof localsObj === "object" ? localsObj : {};
        // Collect enumerable sandbox bindings as locals when not provided.
        if (!localsObj) {
          for (const key of Object.keys(sandbox)) {
            if (key === "parallax" || key === "console") continue;
            err.locals[key] = sandbox[key];
          }
        }
        throw err;
      },
      safepoint(label) {
        return sandbox.parallax.checkpoint(label);
      },
    };
  }
  const context = vm.createContext(sandbox);
  let success = true;
  let suspended = false;
  let exception = null;
  let ues = null;
  let spReport = null;
  const bindings = {};

  // Append a final expression that reads captured names from script scope.
  // Capture names are validated as identifiers above to prevent code injection.
  let fullSource = runSource;
  if (!continuum && capture.length > 0) {
    const parts = capture.map((n) => {
      const key = JSON.stringify(n);
      return `${key}: (function(){ try { return ${n}; } catch (e) { return undefined; } })()`;
    });
    fullSource = `${runSource}\n;({ ${parts.join(", ")} })`;
  }

  const timeoutMs =
    payload.limits && typeof payload.limits.timeout === "number"
      ? payload.limits.timeout
      : 30000;

  try {
    const script = new vm.Script(fullSource, { filename });
    const result = script.runInContext(context, { timeout: timeoutMs });
    if (!continuum && capture.length > 0) {
      for (const name of capture) {
        if (
          result &&
          Object.prototype.hasOwnProperty.call(result, name) &&
          result[name] !== undefined
        ) {
          bindings[name] = encodeValue(result[name]);
        } else {
          success = false;
          exception = {
            type_name: "ReferenceError",
            message: `binding '${name}' not defined after execution`,
            stack: null,
          };
          break;
        }
      }
    }
  } catch (e) {
    if (e && e.parallaxCheckpoint) {
      suspended = true;
      success = true;
      ues = buildUes(e.label, filename, e.locals, resumeSource);
      spReport = safepointReport(e.label);
      Object.assign(bindings, (ues.heap && ues.heap.bindings) || {});
    } else {
      success = false;
      exception = {
        type_name: e.name || "Error",
        message: String(e.message || e),
        stack: e.stack || null,
      };
    }
  }

  const duration_us = Number((process.hrtime.bigint() - t0) / 1000n);
  const out = {
    stdout,
    stderr,
    duration_us,
    bindings,
    exception,
    success,
    suspended,
  };
  if (ues) out.ues = ues;
  if (spReport) out.safepoint = spReport;
  respond(reqId, "execute", true, out);
}

function handleResumeCheckpoint(reqId, payload) {
  const t0 = process.hrtime.bigint();
  const ues = payload.ues || {};
  if (ues.source_runtime && ues.source_runtime !== "javascript") {
    respond(reqId, "resume_checkpoint", false, undefined, {
      code: "CAPABILITY_VIOLATION",
      message: "cross-runtime continuation resume is Unsupported",
      diagnostic: `ues.source_runtime=${ues.source_runtime}`,
    });
    return;
  }
  if (
    ues.format_version !== undefined &&
    Number(ues.format_version) !== UES_FORMAT_VERSION
  ) {
    respond(reqId, "resume_checkpoint", false, undefined, {
      code: "INVALID_SNAPSHOT",
      message: `unsupported UES format version ${ues.format_version}`,
    });
    return;
  }
  const resumeSource = ues.resume_source;
  if (resumeSource === undefined || resumeSource === null) {
    respond(reqId, "resume_checkpoint", false, undefined, {
      code: "UNSUPPORTED_VALUE",
      message: "UES has no resume_source; cannot resume without restarting",
    });
    return;
  }
  if (
    !ues.control_state ||
    ues.control_state.safepoint_kind !== "explicit_checkpoint"
  ) {
    respond(reqId, "resume_checkpoint", false, undefined, {
      code: "UNSUPPORTED_VALUE",
      message: "only explicit_checkpoint UES can be resumed",
    });
    return;
  }

  let bindingsIn = (ues.heap && ues.heap.bindings) || {};
  const stack = ues.call_stack || [];
  if (stack[0] && stack[0].locals) {
    bindingsIn = { ...bindingsIn, ...stack[0].locals };
  }

  let stdout = "";
  let stderr = "";
  const sandbox = {
    console: {
      log: (...args) => {
        stdout += args.map(String).join(" ") + "\n";
      },
      error: (...args) => {
        stderr += args.map(String).join(" ") + "\n";
      },
      warn: (...args) => {
        stderr += args.map(String).join(" ") + "\n";
      },
    },
  };
  const warnings = ["EXPERIMENTAL: same-runtime checkpoint resume"];
  for (const [name, node] of Object.entries(bindingsIn)) {
    if (!isIdent(name)) continue;
    try {
      sandbox[name] = decodeValue(node);
    } catch (e) {
      warnings.push(`skip binding ${name}: ${e.message || e}`);
    }
  }
  const context = vm.createContext(sandbox);
  let success = true;
  let exception = null;
  const timeoutMs =
    payload.limits && typeof payload.limits.timeout === "number"
      ? payload.limits.timeout
      : 30000;
  try {
    if (String(resumeSource).trim()) {
      const script = new vm.Script(resumeSource, {
        filename: ues.source_program || "<resume>",
      });
      script.runInContext(context, { timeout: timeoutMs });
    }
  } catch (e) {
    success = false;
    exception = {
      type_name: e.name || "Error",
      message: String(e.message || e),
      stack: e.stack || null,
    };
  }
  const outBindings = {};
  for (const [k, v] of Object.entries(sandbox)) {
    if (k === "console" || k === "parallax" || !isIdent(k)) continue;
    outBindings[k] = encodeValue(v);
  }
  const duration_us = Number((process.hrtime.bigint() - t0) / 1000n);
  respond(reqId, "resume_checkpoint", true, {
    success,
    stdout,
    stderr,
    duration_us,
    bindings: outBindings,
    exception,
    warnings,
  });
}

function handleRestore(reqId, payload) {
  const t0 = process.hrtime.bigint();
  try {
    const bindingsIn = payload.bindings || {};
    const restored = {};
    const g = {};
    for (const [name, node] of Object.entries(bindingsIn)) {
      g[name] = decodeValue(node);
      restored[name] = typeof g[name];
    }
    const outBindings = {};
    for (const [k, v] of Object.entries(g)) {
      outBindings[k] = encodeValue(v);
    }
    const duration_us = Number((process.hrtime.bigint() - t0) / 1000n);
    respond(reqId, "restore", true, {
      success: true,
      warnings: [],
      restored,
      duration_us,
      bindings: outBindings,
    });
  } catch (e) {
    respond(reqId, "restore", false, undefined, {
      code: "RESTORE_FAILURE",
      message: String(e.message || e),
      diagnostic: e.stack || null,
    });
  }
}

function handleShutdown(reqId) {
  respond(reqId, "shutdown", true, {});
  process.exit(0);
}

const handlers = {
  hello: (id, p) => handleHello(id, p),
  execute: handleExecute,
  restore: handleRestore,
  resume_checkpoint: handleResumeCheckpoint,
  shutdown: handleShutdown,
  ping: (id) => respond(id, "ping", true, { pong: true }),
};

const rl = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
rl.on("line", (line) => {
  line = line.trim();
  if (!line) return;
  let msg;
  try {
    msg = JSON.parse(line);
  } catch (e) {
    process.stderr.write(`protocol error: ${e}\n`);
    return;
  }
  const reqId = msg.id;
  const op = msg.op;
  const payload = msg.payload || {};
  if (msg.v !== undefined && Number(msg.v) !== PROTOCOL_VERSION) {
    respond(reqId, op || "unknown", false, undefined, {
      code: "PROTOCOL_VIOLATION",
      message: `protocol version mismatch: got ${msg.v}, expected ${PROTOCOL_VERSION}`,
    });
    return;
  }
  const handler = handlers[op];
  if (!handler) {
    respond(reqId, op || "unknown", false, undefined, {
      code: "PROTOCOL_VIOLATION",
      message: `unknown op: ${op}`,
    });
    return;
  }
  try {
    handler(reqId, payload);
  } catch (e) {
    respond(reqId, op, false, undefined, {
      code: "INTERNAL",
      message: String(e.message || e),
      diagnostic: e.stack || null,
    });
  }
});
