// Parallax Go runtime worker — NDJSON on stdin/stdout.
// Experimental: execute guest programs via `go run` on a temp package; no binding migrate.
package main

import (
	"bufio"
	"bytes"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"time"
)

const (
	protocolVersion = 1
	adapterVersion  = "0.1.0"
)

type envelope struct {
	V       uint32          `json:"v"`
	ID      string          `json:"id"`
	Op      string          `json:"op"`
	OK      *bool           `json:"ok,omitempty"`
	Payload json.RawMessage `json:"payload,omitempty"`
	Error   *protoErr       `json:"error,omitempty"`
}

type protoErr struct {
	Code    string `json:"code"`
	Message string `json:"message"`
}

func reply(id, op string, ok bool, payload any, err *protoErr) {
	env := map[string]any{
		"v":  protocolVersion,
		"id": id,
		"op": op,
		"ok": ok,
	}
	if payload != nil {
		env["payload"] = payload
	}
	if err != nil {
		env["error"] = err
	}
	b, _ := json.Marshal(env)
	os.Stdout.Write(append(b, '\n'))
}

func handleHello(id string) {
	reply(id, "hello", true, map[string]any{
		"protocol_version": protocolVersion,
		"runtime":          map[string]string{"other": "go"},
		"host_version":     runtime.Version(),
		"adapter_version":  adapterVersion,
	}, nil)
}

func handleExecute(id string, payload json.RawMessage) {
	var req struct {
		Source   string   `json:"source"`
		Filename string   `json:"filename"`
		Capture  []string `json:"capture"`
	}
	_ = json.Unmarshal(payload, &req)
	t0 := time.Now()
	dir, err := os.MkdirTemp("", "plx-go-*")
	if err != nil {
		reply(id, "execute", false, nil, &protoErr{Code: "IO", Message: err.Error()})
		return
	}
	defer os.RemoveAll(dir)

	src := req.Source
	// Ensure package main + main() for snippets that are bare statements.
	if !strings.Contains(src, "package ") {
		src = "package main\n\nimport \"fmt\"\n\nfunc main() {\n" + src + "\n}\n"
	}
	path := filepath.Join(dir, "main.go")
	if err := os.WriteFile(path, []byte(src), 0o600); err != nil {
		reply(id, "execute", false, nil, &protoErr{Code: "IO", Message: err.Error()})
		return
	}
	cmd := exec.Command("go", "run", path)
	cmd.Dir = dir
	var stdout, stderr bytes.Buffer
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr
	runErr := cmd.Run()
	success := runErr == nil
	var exception any
	if runErr != nil {
		exception = map[string]string{
			"type_name": "GoRunError",
			"message":   runErr.Error(),
		}
	}
	duration := time.Since(t0).Microseconds()
	reply(id, "execute", true, map[string]any{
		"stdout":      stdout.String(),
		"stderr":      stderr.String(),
		"duration_us": duration,
		"bindings":    map[string]any{},
		"exception":   exception,
		"success":     success,
		"suspended":   false,
	}, nil)
}

func handleRestore(id string, _ json.RawMessage) {
	reply(id, "restore", false, nil, &protoErr{
		Code:    "UNSUPPORTED_VALUE",
		Message: "Go connector is execute-only experimental — binding restore not implemented",
	})
}

func handleShutdown(id string) {
	reply(id, "shutdown", true, map[string]any{}, nil)
	os.Exit(0)
}

func main() {
	sc := bufio.NewScanner(os.Stdin)
	// Allow large execute payloads.
	buf := make([]byte, 0, 64*1024)
	sc.Buffer(buf, 16*1024*1024)
	for sc.Scan() {
		line := strings.TrimSpace(sc.Text())
		if line == "" {
			continue
		}
		var env envelope
		if err := json.Unmarshal([]byte(line), &env); err != nil {
			fmt.Fprintln(os.Stderr, "bad json:", err)
			continue
		}
		switch env.Op {
		case "hello":
			handleHello(env.ID)
		case "execute":
			handleExecute(env.ID, env.Payload)
		case "restore":
			handleRestore(env.ID, env.Payload)
		case "shutdown":
			handleShutdown(env.ID)
		default:
			reply(env.ID, env.Op, false, nil, &protoErr{
				Code:    "UNSUPPORTED_VALUE",
				Message: "unknown op: " + env.Op,
			})
		}
	}
}
