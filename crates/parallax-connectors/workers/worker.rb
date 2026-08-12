#!/usr/bin/env ruby
# frozen_string_literal: true
# Parallax Ruby runtime worker — NDJSON on stdin/stdout.

require "json"
require "base64"
require "stringio"

PROTOCOL_VERSION = 1
ADAPTER_VERSION = "0.1.0"

def encode_value(obj)
  case obj
  when nil then { "t" => "null" }
  when TrueClass, FalseClass then { "t" => "bool", "v" => obj }
  when Integer then { "t" => "int", "v" => { "decimal" => obj.to_s } }
  when Float
    if obj.nan?
      { "t" => "float", "v" => "NaN" }
    elsif obj.infinite?
      { "t" => "float", "v" => (obj > 0 ? "Infinity" : "-Infinity") }
    else
      { "t" => "float", "v" => obj }
    end
  when String then { "t" => "string", "v" => obj }
  when Symbol then { "t" => "string", "v" => obj.to_s }
  when Array then { "t" => "list", "v" => obj.map { |x| encode_value(x) } }
  when Hash
    entries = obj.map { |k, v| { "key" => encode_value(k), "value" => encode_value(v) } }
    { "t" => "map", "entries" => entries }
  else
    {
      "t" => "unsupported",
      "reason" => "unsupported Ruby type: #{obj.class}",
      "repr" => obj.inspect,
      "type_name" => obj.class.name
    }
  end
end

def decode_value(node)
  raise "invalid PIR node" unless node.is_a?(Hash) && node["t"]
  case node["t"]
  when "null" then nil
  when "bool" then !!node["v"]
  when "int"
    v = node["v"]
    v.is_a?(Hash) ? Integer(v["decimal"]) : Integer(v)
  when "float"
    v = node["v"]
    case v
    when "NaN" then Float::NAN
    when "Infinity", "+Infinity" then Float::INFINITY
    when "-Infinity" then -Float::INFINITY
    else Float(v)
    end
  when "string" then node["v"].to_s
  when "list" then Array(node["v"]).map { |x| decode_value(x) }
  when "map"
    out = {}
    Array(node["entries"]).each do |e|
      out[decode_value(e["key"])] = decode_value(e["value"])
    end
    out
  when "bigint", "big_int" then Integer(node["v"])
  else
    raise "cannot restore PIR type #{node['t']}"
  end
end

def reply(id, op, ok:, payload: nil, error: nil)
  env = { "v" => PROTOCOL_VERSION, "id" => id, "op" => op, "ok" => ok }
  env["payload"] = payload unless payload.nil?
  env["error"] = error unless error.nil?
  $stdout.write(JSON.generate(env) + "\n")
  $stdout.flush
end

def handle_hello(id, payload)
  reply(id, "hello", ok: true, payload: {
    "protocol_version" => PROTOCOL_VERSION,
    "runtime" => { "other" => "ruby" },
    "host_version" => RUBY_VERSION,
    "adapter_version" => ADAPTER_VERSION
  })
end

def handle_execute(id, payload)
  source = payload["source"].to_s
  capture = Array(payload["capture"])
  t0 = Process.clock_gettime(Process::CLOCK_MONOTONIC)
  stdout_buf = StringIO.new
  stderr_buf = StringIO.new
  success = true
  exception = nil
  bindings = {}
  begin
    old_out = $stdout
    old_err = $stderr
    $stdout = stdout_buf
    $stderr = stderr_buf
    # Evaluate in an isolated binding so we can capture locals.
    b = binding
    b.eval(source, payload["filename"] || "(parallax)")
    capture.each do |name|
      next unless name =~ /\A[A-Za-z_][A-Za-z0-9_]*\z/
      begin
        bindings[name] = encode_value(b.local_variable_get(name))
      rescue NameError
        # try constants / top-level ivars lightly
        begin
          bindings[name] = encode_value(b.eval(name))
        rescue StandardError
          # skip missing
        end
      end
    end
  rescue StandardError => e
    success = false
    exception = { "type_name" => e.class.name, "message" => e.message }
  ensure
    $stdout = old_out
    $stderr = old_err
  end
  duration_us = ((Process.clock_gettime(Process::CLOCK_MONOTONIC) - t0) * 1_000_000).to_i
  reply(id, "execute", ok: true, payload: {
    "stdout" => stdout_buf.string,
    "stderr" => stderr_buf.string,
    "duration_us" => duration_us,
    "bindings" => bindings,
    "exception" => exception,
    "success" => success,
    "suspended" => false
  })
end

def handle_restore(id, payload)
  bindings = payload["bindings"] || {}
  restored = {}
  bindings.each do |name, node|
    next unless name =~ /\A[A-Za-z_][A-Za-z0-9_]*\z/
    begin
      decode_value(node)
      restored[name] = "ok"
    rescue StandardError => e
      reply(id, "restore", ok: false, error: {
        "code" => "UNSUPPORTED_VALUE",
        "message" => "restore #{name}: #{e.message}"
      })
      return
    end
  end
  reply(id, "restore", ok: true, payload: {
    "success" => true,
    "warnings" => [],
    "restored" => restored,
    "duration_us" => 0
  })
end

def handle_shutdown(id, _payload)
  reply(id, "shutdown", ok: true, payload: {})
  exit 0
end

STDIN.each_line do |line|
  line = line.strip
  next if line.empty?
  begin
    env = JSON.parse(line)
  rescue JSON::ParserError => e
    warn "bad json: #{e}"
    next
  end
  id = env["id"]
  op = env["op"]
  payload = env["payload"] || {}
  begin
    case op
    when "hello" then handle_hello(id, payload)
    when "execute" then handle_execute(id, payload)
    when "restore" then handle_restore(id, payload)
    when "shutdown" then handle_shutdown(id, payload)
    else
      reply(id, op, ok: false, error: {
        "code" => "UNSUPPORTED_VALUE",
        "message" => "unknown op: #{op}"
      })
    end
  rescue StandardError => e
    reply(id, op, ok: false, error: {
      "code" => "ADAPTER_CRASHED",
      "message" => e.message
    })
  end
end
