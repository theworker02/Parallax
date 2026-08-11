//! Differential execution for behavioral equivalence (testing confidence, not proof).

use parallax_core::{ErrorCode, ParallaxError};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DifferentialCase {
    pub name: String,
    pub inputs_json: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DifferentialResult {
    pub name: String,
    pub matched: bool,
    pub source_output: String,
    pub target_output: String,
    pub detail: String,
}

pub struct DifferentialRunner;

impl DifferentialRunner {
    /// Run cargo test in target as a proxy differential suite when present.
    /// For pure functions, prefer calling into migrated unit tests.
    pub fn verify_target_tests(target: &Path) -> Result<Vec<DifferentialResult>, ParallaxError> {
        if !target.join("Cargo.toml").exists() {
            return Err(ParallaxError::new(
                ErrorCode::UnsupportedValue,
                "differential verify currently requires a Rust target with Cargo.toml",
            )
            .with_source("parallax-mirror"));
        }
        let out = Command::new("cargo")
            .args(["test", "--", "--nocapture"])
            .current_dir(target)
            .env_remove("RUSTFLAGS")
            .env_remove("RUSTDOCFLAGS")
            .output()
            .map_err(|e| {
                ParallaxError::new(ErrorCode::ExecutionFailure, e.to_string())
                    .with_source("parallax-mirror")
            })?;
        let text = format!(
            "{}\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let ok = out.status.success();
        Ok(vec![DifferentialResult {
            name: "cargo_test_suite".into(),
            matched: ok,
            source_output: "see source project tests".into(),
            target_output: text
                .chars()
                .rev()
                .take(1500)
                .collect::<String>()
                .chars()
                .rev()
                .collect(),
            detail: if ok {
                "target test suite passed (behavioral confidence via migrated tests — not formal proof)"
                    .into()
            } else {
                "target test suite failed".into()
            },
        }])
    }

    /// Generate simple boundary inputs for a numeric signature (documentation helper).
    pub fn numeric_boundary_cases(fn_name: &str) -> Vec<DifferentialCase> {
        vec![0i64, 1, -1, i32::MAX as i64, i32::MIN as i64]
            .into_iter()
            .enumerate()
            .map(|(i, v)| DifferentialCase {
                name: format!("{fn_name}_boundary_{i}"),
                inputs_json: serde_json::json!([v]),
            })
            .collect()
    }
}
