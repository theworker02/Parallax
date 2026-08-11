//! Source frontend orchestration (TypeScript via Node + TypeScript compiler).

use parallax_core::{ErrorCode, ParallaxError, Remediation};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Whether path looks like a project root (directory with manifests / src).
pub fn is_project_root(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    path.join("package.json").exists()
        || path.join("pyproject.toml").exists()
        || path.join("Cargo.toml").exists()
        || path.join("go.mod").exists()
        || path.join("tsconfig.json").exists()
        || path.join("src").is_dir()
}

/// Locate the TypeScript analyzer script.
pub fn typescript_analyzer_script() -> Result<PathBuf, ParallaxError> {
    let mut candidates = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("adapters/typescript/analyze.mjs"));
    }
    candidates.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../adapters/typescript/analyze.mjs"),
    );
    candidates.push(PathBuf::from("adapters/typescript/analyze.mjs"));
    for c in candidates {
        if c.is_file() {
            // Prefer a normalized path without the Windows `\\?\` prefix — Node can
            // mishandle verbatim paths that contain spaces.
            let normalized = c
                .canonicalize()
                .map(|p| {
                    let s = p.to_string_lossy();
                    if let Some(stripped) = s.strip_prefix(r"\\?\") {
                        PathBuf::from(stripped)
                    } else {
                        p
                    }
                })
                .unwrap_or(c);
            return Ok(normalized);
        }
    }
    Err(ParallaxError::new(
        ErrorCode::RuntimeUnavailable,
        "TypeScript analyzer script not found (adapters/typescript/analyze.mjs)",
    )
    .with_source("parallax-transmute")
    .with_operation("typescript_analyzer_script")
    .remediate(Remediation::new(
        "Run from the Parallax repository root so adapters/typescript is available",
    )))
}

/// Run the Node TypeScript frontend; returns JSON text for ProjectAnalysis-shaped payload.
pub fn run_typescript_frontend(root: &Path) -> Result<String, ParallaxError> {
    let script = typescript_analyzer_script()?;
    // Ensure local typescript package is present
    let adapter_dir = script.parent().unwrap();
    if !adapter_dir.join("node_modules/typescript").exists() {
        let npm = Command::new("npm")
            .args(["install", "--no-fund", "--no-audit"])
            .current_dir(adapter_dir)
            .output();
        if let Ok(out) = npm {
            if !out.status.success() {
                return Err(ParallaxError::new(
                    ErrorCode::RuntimeInitializationFailure,
                    format!(
                        "npm install failed in adapters/typescript: {}",
                        String::from_utf8_lossy(&out.stderr)
                    ),
                )
                .with_source("parallax-transmute")
                .remediate(Remediation::new(
                    "Install Node.js and run npm install in adapters/typescript",
                )));
            }
        }
    }
    let root_arg = {
        let s = root.to_string_lossy();
        PathBuf::from(s.strip_prefix(r"\\?\").unwrap_or(&s))
    };
    let out = Command::new("node")
        .arg(&script)
        .arg(&root_arg)
        .output()
        .map_err(|e| {
            ParallaxError::new(
                ErrorCode::RuntimeUnavailable,
                format!("failed to spawn node for TypeScript frontend: {e}"),
            )
            .with_source("parallax-transmute")
            .with_operation("run_typescript_frontend")
            .remediate(Remediation::new("Install Node.js and ensure it is on PATH"))
        })?;
    if !out.status.success() {
        return Err(ParallaxError::new(
            ErrorCode::CaptureFailure,
            format!(
                "TypeScript frontend failed: {}",
                String::from_utf8_lossy(&out.stderr)
            ),
        )
        .with_source("parallax-transmute")
        .with_operation("run_typescript_frontend")
        .with_diagnostic(String::from_utf8_lossy(&out.stdout)));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}
