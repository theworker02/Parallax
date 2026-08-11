//! End-to-end migration tests (require host runtimes).

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn plx() -> Command {
    Command::cargo_bin("plx").unwrap()
}

#[test]
fn doctor_runs() {
    let mut cmd = plx();
    cmd.current_dir(repo_root()).arg("doctor").arg("--json");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("parallax_version"))
        .stdout(predicate::str::contains("adapter_interface"));
}

#[test]
fn version_lists_all_components() {
    let mut cmd = plx();
    cmd.current_dir(repo_root())
        .args(["version", "--format", "json"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("pir_schema"))
        .stdout(predicate::str::contains("protocol"))
        .stdout(predicate::str::contains("snapshot"))
        .stdout(predicate::str::contains("adapter_interface"));
}

#[test]
fn migrate_to_wasm_is_structured_unsupported() {
    let mut cmd = plx();
    cmd.current_dir(repo_root())
        .args(["migrate", "examples/demo.py", "--to", "wasm", "--json"]);
    let assert = cmd.assert().failure();
    let output = assert.get_output();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let blob = format!("{stderr}{stdout}");
    assert!(
        blob.contains("CapabilityViolation")
            || blob.contains("CAPABILITY_VIOLATION")
            || blob.contains("unsupported"),
        "expected structured capability error, got: {blob}"
    );
}

#[test]
fn migrate_python_to_js_when_python_available() {
    let root = repo_root();
    let mut doctor = plx();
    let out = doctor
        .current_dir(&root)
        .args(["doctor", "--json"])
        .output()
        .expect("doctor");
    let stdout = String::from_utf8_lossy(&out.stdout);
    if !stdout.contains("\"python\"") || !stdout.contains("\"ready\"") {
        // Skip soft: Python may be unavailable on some hosts.
        eprintln!("skipping migrate test: python not ready\n{stdout}");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let out_js = tmp.path().join("demo.migrated.js");
    let mut cmd = plx();
    cmd.current_dir(&root)
        .args(["migrate", "examples/demo.py", "--to", "javascript", "-o"])
        .arg(&out_js)
        .arg("--json");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("username"))
        .stdout(predicate::str::contains("Ada"));
    let emitted = std::fs::read_to_string(&out_js).unwrap();
    assert!(emitted.contains("Ada"));
    assert!(emitted.contains("compiler"));
}

#[test]
fn migrate_js_to_python_when_both_available() {
    let root = repo_root();
    let mut doctor = plx();
    let out = doctor
        .current_dir(&root)
        .args(["doctor", "--json"])
        .output()
        .expect("doctor");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let py_ready = stdout.contains("\"python\"") && stdout.contains("\"ready\"");
    let js_ready = stdout.contains("\"javascript\"") && stdout.contains("\"ready\"");
    if !(py_ready && js_ready) {
        eprintln!("skipping reverse migrate: runtimes not ready\n{stdout}");
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let out_py = tmp.path().join("demo.migrated.py");
    let mut cmd = plx();
    cmd.current_dir(&root)
        .args(["migrate", "examples/demo.js", "--to", "python", "-o"])
        .arg(&out_py);
    cmd.assert().success();
    let emitted = std::fs::read_to_string(&out_py).unwrap();
    assert!(emitted.contains("Ada"));
}
