//! First-class error taxonomy for Parallax.

use crate::runtime::RuntimeKind;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Stable machine-readable error codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// Host runtime binary/engine is missing or not discoverable.
    RuntimeUnavailable,
    /// Adapter failed during initialization.
    RuntimeInitializationFailure,
    /// Guest program failed during execution.
    ExecutionFailure,
    /// Execution exceeded its deadline.
    ExecutionTimeout,
    /// Execution was cancelled by the caller.
    ExecutionCancelled,
    /// State capture failed.
    CaptureFailure,
    /// State restore failed.
    RestoreFailure,
    /// Serialization of PIR/snapshot/protocol failed.
    SerializationFailure,
    /// Snapshot failed integrity or schema validation.
    InvalidSnapshot,
    /// Worker protocol violation or version mismatch.
    ProtocolViolation,
    /// Requested capability is not available.
    CapabilityViolation,
    /// Migration was rejected by policy or analysis.
    MigrationRejected,
    /// Semantic loss detected (often with MigrationRejected).
    SemanticLoss,
    /// Value cannot be represented in PIR or target runtime.
    UnsupportedValue,
    /// Runtime worker process crashed.
    AdapterCrashed,
    /// Resource limit exceeded (memory, output, message size).
    ResourceLimitExceeded,
    /// Generic internal error — should be rare.
    Internal,
    /// Invalid user input / CLI arguments.
    InvalidArgument,
    /// I/O failure.
    Io,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_else(|| format!("{:?}", self));
        write!(f, "{}", s)
    }
}

/// Suggested remediation for an error.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Remediation {
    /// Short actionable suggestion.
    pub action: String,
    /// Optional detail.
    pub detail: Option<String>,
}

impl Remediation {
    /// Create a simple remediation.
    pub fn new(action: impl Into<String>) -> Self {
        Self {
            action: action.into(),
            detail: None,
        }
    }

    /// Create a remediation with detail.
    pub fn with_detail(action: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            action: action.into(),
            detail: Some(detail.into()),
        }
    }
}

/// Structured Parallax error.
#[derive(Clone, Debug, Error, Serialize, Deserialize)]
#[error("{code}: {message}")]
pub struct ParallaxError {
    /// Stable error code.
    pub code: ErrorCode,
    /// Human-readable message.
    pub message: String,
    /// Component that produced the error.
    pub source_component: String,
    /// Operation being attempted.
    pub operation: String,
    /// Optional runtime involved.
    pub runtime: Option<RuntimeKind>,
    /// Structured context key/value pairs.
    pub context: Vec<(String, String)>,
    /// Suggested remediations.
    pub remediations: Vec<Remediation>,
    /// Optional verbose diagnostic (stack, stderr, etc.).
    pub diagnostic: Option<String>,
}

impl ParallaxError {
    /// Start building an error.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            source_component: "parallax".into(),
            operation: "unknown".into(),
            runtime: None,
            context: Vec::new(),
            remediations: Vec::new(),
            diagnostic: None,
        }
    }

    /// Set source component.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source_component = source.into();
        self
    }

    /// Set operation name.
    pub fn with_operation(mut self, op: impl Into<String>) -> Self {
        self.operation = op.into();
        self
    }

    /// Attach runtime.
    pub fn with_runtime(mut self, runtime: RuntimeKind) -> Self {
        self.runtime = Some(runtime);
        self
    }

    /// Add context pair.
    pub fn context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.push((key.into(), value.into()));
        self
    }

    /// Add remediation.
    pub fn remediate(mut self, remediation: Remediation) -> Self {
        self.remediations.push(remediation);
        self
    }

    /// Attach verbose diagnostic.
    pub fn with_diagnostic(mut self, diagnostic: impl Into<String>) -> Self {
        self.diagnostic = Some(diagnostic.into());
        self
    }

    /// Format a concise user-facing report (non-JSON).
    pub fn format_report(&self, verbose: bool) -> String {
        let mut out = String::new();
        out.push_str(&format!("{}: {}\n", self.code, self.message));
        if let Some(rt) = &self.runtime {
            out.push_str(&format!("Runtime: {}\n", rt));
        }
        out.push_str(&format!("Operation: {}\n", self.operation));
        for (k, v) in &self.context {
            out.push_str(&format!("{}:\n  {}\n", k, v));
        }
        if !self.remediations.is_empty() {
            out.push_str("Possible actions:\n");
            for r in &self.remediations {
                out.push_str(&format!("  • {}\n", r.action));
                if let Some(d) = &r.detail {
                    out.push_str(&format!("    {}\n", d));
                }
            }
        }
        if verbose {
            if let Some(diag) = &self.diagnostic {
                out.push_str("\nDiagnostic:\n");
                out.push_str(diag);
                out.push('\n');
            }
        }
        out
    }
}

/// Convenient result alias.
pub type Result<T> = std::result::Result<T, ParallaxError>;

impl From<std::io::Error> for ParallaxError {
    fn from(err: std::io::Error) -> Self {
        ParallaxError::new(ErrorCode::Io, err.to_string())
            .with_source("std::io")
            .with_operation("io")
    }
}

impl From<serde_json::Error> for ParallaxError {
    fn from(err: serde_json::Error) -> Self {
        ParallaxError::new(ErrorCode::SerializationFailure, err.to_string())
            .with_source("serde_json")
            .with_operation("serialize")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_includes_remediation() {
        let err = ParallaxError::new(ErrorCode::MigrationRejected, "integer precision")
            .context("Value", "9007199254740993")
            .remediate(Remediation::new("migrate using BigInt"));
        let report = err.format_report(false);
        assert!(report.contains("PYTHON_INT_PRECISION") || report.contains("MIGRATION_REJECTED"));
        assert!(report.contains("BigInt"));
    }
}
