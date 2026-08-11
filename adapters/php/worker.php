#!/usr/bin/env php
<?php
/**
 * Parallax PHP runtime worker — NDJSON on stdin/stdout.
 */

const PROTOCOL_VERSION = 1;
const ADAPTER_VERSION = "0.1.0";

function encode_value($obj) {
    if ($obj === null) {
        return ["t" => "null"];
    }
    if (is_bool($obj)) {
        return ["t" => "bool", "v" => $obj];
    }
    if (is_int($obj)) {
        return ["t" => "int", "v" => ["decimal" => (string)$obj]];
    }
    if (is_float($obj)) {
        if (is_nan($obj)) {
            return ["t" => "float", "v" => "NaN"];
        }
        if (is_infinite($obj)) {
            return ["t" => "float", "v" => $obj > 0 ? "Infinity" : "-Infinity"];
        }
        return ["t" => "float", "v" => $obj];
    }
    if (is_string($obj)) {
        return ["t" => "string", "v" => $obj];
    }
    if (is_array($obj)) {
        $isList = array_keys($obj) === range(0, count($obj) - 1);
        if ($isList) {
            return ["t" => "list", "v" => array_map("encode_value", $obj)];
        }
        $entries = [];
        foreach ($obj as $k => $v) {
            $entries[] = ["key" => encode_value($k), "value" => encode_value($v)];
        }
        return ["t" => "map", "entries" => $entries];
    }
    return [
        "t" => "unsupported",
        "reason" => "unsupported PHP type: " . gettype($obj),
        "repr" => @json_encode($obj),
        "type_name" => gettype($obj),
    ];
}

function decode_value($node) {
    if (!is_array($node) || !isset($node["t"])) {
        throw new Exception("invalid PIR node");
    }
    switch ($node["t"]) {
        case "null":
            return null;
        case "bool":
            return (bool)$node["v"];
        case "int":
            $v = $node["v"];
            return is_array($v) ? intval($v["decimal"]) : intval($v);
        case "float":
            $v = $node["v"];
            if ($v === "NaN") {
                return NAN;
            }
            if ($v === "Infinity" || $v === "+Infinity") {
                return INF;
            }
            if ($v === "-Infinity") {
                return -INF;
            }
            return floatval($v);
        case "string":
            return (string)$node["v"];
        case "list":
            return array_map("decode_value", $node["v"] ?? []);
        case "map":
            $out = [];
            foreach ($node["entries"] ?? [] as $e) {
                $out[decode_value($e["key"])] = decode_value($e["value"]);
            }
            return $out;
        case "bigint":
        case "big_int":
            return intval($node["v"]);
        default:
            throw new Exception("cannot restore PIR type " . $node["t"]);
    }
}

function reply($id, $op, $ok, $payload = null, $error = null) {
    $env = ["v" => PROTOCOL_VERSION, "id" => $id, "op" => $op, "ok" => $ok];
    if ($payload !== null) {
        $env["payload"] = $payload;
    }
    if ($error !== null) {
        $env["error"] = $error;
    }
    fwrite(STDOUT, json_encode($env) . "\n");
    fflush(STDOUT);
}

function handle_hello($id, $payload) {
    reply($id, "hello", true, [
        "protocol_version" => PROTOCOL_VERSION,
        "runtime" => ["other" => "php"],
        "host_version" => PHP_VERSION,
        "adapter_version" => ADAPTER_VERSION,
    ]);
}

function handle_execute($id, $payload) {
    $source = (string)($payload["source"] ?? "");
    $capture = $payload["capture"] ?? [];
    $filename = $payload["filename"] ?? "(parallax)";
    $t0 = hrtime(true);
    $stdout = "";
    $stderr = "";
    $success = true;
    $exception = null;
    $bindings = [];
    try {
        ob_start();
        // Isolate symbols in a local scope via include of a temp file for capture.
        $tmp = tempnam(sys_get_temp_dir(), "plxphp");
        file_put_contents($tmp, "<?php\n" . $source);
        $__plx_scope = [];
        // Capture declared variables by extracting after include.
        include $tmp;
        @unlink($tmp);
        $stdout = ob_get_clean();
        foreach ($capture as $name) {
            if (!preg_match('/^[A-Za-z_][A-Za-z0-9_]*$/', $name)) {
                continue;
            }
            if (isset($$name)) {
                $bindings[$name] = encode_value($$name);
            }
        }
    } catch (Throwable $e) {
        if (ob_get_level() > 0) {
            $stdout = ob_get_clean();
        }
        $success = false;
        $exception = ["type_name" => get_class($e), "message" => $e->getMessage()];
    }
    $duration_us = intdiv(hrtime(true) - $t0, 1000);
    reply($id, "execute", true, [
        "stdout" => $stdout,
        "stderr" => $stderr,
        "duration_us" => $duration_us,
        "bindings" => (object)$bindings,
        "exception" => $exception,
        "success" => $success,
        "suspended" => false,
    ]);
}

function handle_restore($id, $payload) {
    $bindings = $payload["bindings"] ?? [];
    $restored = [];
    $t0 = hrtime(true);
    foreach ($bindings as $name => $node) {
        if (!preg_match('/^[A-Za-z_][A-Za-z0-9_]*$/', $name)) {
            continue;
        }
        try {
            decode_value($node);
            $restored[$name] = "ok";
        } catch (Throwable $e) {
            reply($id, "restore", false, null, [
                "code" => "UNSUPPORTED_VALUE",
                "message" => "restore $name: " . $e->getMessage(),
            ]);
            return;
        }
    }
    reply($id, "restore", true, [
        "success" => true,
        "warnings" => [],
        "restored" => (object)$restored,
        "duration_us" => intdiv(hrtime(true) - $t0, 1000),
    ]);
}

function handle_shutdown($id, $payload) {
    reply($id, "shutdown", true, new stdClass());
    exit(0);
}

stream_set_blocking(STDIN, true);
while (($line = fgets(STDIN)) !== false) {
    $line = trim($line);
    if ($line === "") {
        continue;
    }
    $env = json_decode($line, true);
    if (!is_array($env)) {
        continue;
    }
    $id = $env["id"] ?? null;
    $op = $env["op"] ?? "";
    $payload = $env["payload"] ?? [];
    try {
        switch ($op) {
            case "hello":
                handle_hello($id, $payload);
                break;
            case "execute":
                handle_execute($id, $payload);
                break;
            case "restore":
                handle_restore($id, $payload);
                break;
            case "shutdown":
                handle_shutdown($id, $payload);
                break;
            default:
                reply($id, $op, false, null, [
                    "code" => "UNSUPPORTED_VALUE",
                    "message" => "unknown op: $op",
                ]);
        }
    } catch (Throwable $e) {
        reply($id, $op, false, null, [
            "code" => "ADAPTER_CRASHED",
            "message" => $e->getMessage(),
        ]);
    }
}
