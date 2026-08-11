//! Versioned NDJSON worker protocol for Parallax adapters.

#![deny(unsafe_code)]
#![warn(missing_docs)]

use parallax_core::{
    check_protocol, ErrorCode, ExecutionLimits, ParallaxError, RequestId, RuntimeKind,
    PROTOCOL_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Protocol version re-export.
pub const VERSION: u32 = PROTOCOL_VERSION;

/// Envelope wrapping every worker message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Envelope {
    /// Protocol version.
    pub v: u32,
    /// Correlation id.
    pub id: RequestId,
    /// Operation name.
    pub op: String,
    /// Success flag (responses only; omitted on requests).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ok: Option<bool>,
    /// Payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,
    /// Structured error (responses only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ProtocolError>,
}

impl Envelope {
    /// Build a request envelope.
    pub fn request(op: impl Into<String>, payload: Value) -> Self {
        Self {
            v: VERSION,
            id: RequestId::new(),
            op: op.into(),
            ok: None,
            payload: Some(payload),
            error: None,
        }
    }

    /// Build a successful response.
    pub fn ok_response(id: RequestId, op: impl Into<String>, payload: Value) -> Self {
        Self {
            v: VERSION,
            id,
            op: op.into(),
            ok: Some(true),
            payload: Some(payload),
            error: None,
        }
    }

    /// Build an error response.
    pub fn err_response(id: RequestId, op: impl Into<String>, error: ProtocolError) -> Self {
        Self {
            v: VERSION,
            id,
            op: op.into(),
            ok: Some(false),
            payload: None,
            error: Some(error),
        }
    }

    /// Encode as a single NDJSON line (with trailing newline).
    pub fn to_ndjson_line(&self) -> parallax_core::Result<String> {
        let mut s = serde_json::to_string(self).map_err(ParallaxError::from)?;
        s.push('\n');
        Ok(s)
    }

    /// Parse a single NDJSON line.
    pub fn from_ndjson_line(line: &str) -> parallax_core::Result<Self> {
        let env: Self = serde_json::from_str(line.trim()).map_err(|e| {
            ParallaxError::new(ErrorCode::ProtocolViolation, e.to_string())
                .with_source("parallax-protocol")
                .with_operation("from_ndjson_line")
        })?;
        check_protocol(env.v).map_err(|e| {
            e.with_source("parallax-protocol")
                .with_operation("from_ndjson_line")
        })?;
        Ok(env)
    }
}

/// Protocol-level error payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProtocolError {
    /// Error code string.
    pub code: String,
    /// Message.
    pub message: String,
    /// Optional diagnostic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<String>,
}

impl ProtocolError {
    /// From a ParallaxError.
    pub fn from_parallax(err: &ParallaxError) -> Self {
        Self {
            code: err.code.to_string(),
            message: err.message.clone(),
            diagnostic: err.diagnostic.clone(),
        }
    }

    /// Convert to ParallaxError.
    pub fn into_parallax(self, runtime: Option<RuntimeKind>) -> ParallaxError {
        let code = match self.code.as_str() {
            "RUNTIME_UNAVAILABLE" => ErrorCode::RuntimeUnavailable,
            "EXECUTION_FAILURE" => ErrorCode::ExecutionFailure,
            "EXECUTION_TIMEOUT" => ErrorCode::ExecutionTimeout,
            "EXECUTION_CANCELLED" => ErrorCode::ExecutionCancelled,
            "CAPTURE_FAILURE" => ErrorCode::CaptureFailure,
            "RESTORE_FAILURE" => ErrorCode::RestoreFailure,
            "UNSUPPORTED_VALUE" => ErrorCode::UnsupportedValue,
            "PROTOCOL_VIOLATION" => ErrorCode::ProtocolViolation,
            "RESOURCE_LIMIT_EXCEEDED" => ErrorCode::ResourceLimitExceeded,
            "ADAPTER_CRASHED" => ErrorCode::AdapterCrashed,
            _ => ErrorCode::Internal,
        };
        let mut err = ParallaxError::new(code, self.message)
            .with_source("worker")
            .with_operation("protocol");
        if let Some(rt) = runtime {
            err = err.with_runtime(rt);
        }
        if let Some(d) = self.diagnostic {
            err = err.with_diagnostic(d);
        }
        err
    }
}

/// Hello request payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HelloRequest {
    /// Core protocol version.
    pub protocol_version: u32,
    /// Expected runtime kind.
    pub runtime: RuntimeKind,
}

/// Hello response payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HelloResponse {
    /// Worker protocol version.
    pub protocol_version: u32,
    /// Runtime kind.
    pub runtime: RuntimeKind,
    /// Host language/runtime version string.
    pub host_version: String,
    /// Adapter version.
    pub adapter_version: String,
}

/// Validate a hello response against the host protocol version.
pub fn validate_hello(resp: &HelloResponse) -> parallax_core::Result<()> {
    check_protocol(resp.protocol_version).map_err(|e| {
        e.with_source("parallax-protocol")
            .with_operation("validate_hello")
    })
}

/// Execute request payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecuteRequest {
    /// Source code.
    pub source: String,
    /// Filename hint.
    pub filename: String,
    /// Binding names to capture after execution.
    #[serde(default)]
    pub capture: Vec<String>,
    /// Resource limits.
    pub limits: ExecutionLimits,
    /// Enable Continuum safepoint / checkpoint capture (explicit `parallax.checkpoint`).
    #[serde(default)]
    pub continuum: bool,
}

/// Execute response payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecuteResponse {
    /// Captured stdout.
    pub stdout: String,
    /// Captured stderr.
    pub stderr: String,
    /// Wall duration microseconds.
    pub duration_us: u64,
    /// Captured bindings as PIR-tagged JSON object.
    #[serde(default)]
    pub bindings: Value,
    /// Optional exception.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exception: Option<Value>,
    /// Success flag for guest code.
    pub success: bool,
    /// True when paused at an explicit Continuum safepoint (not a failure).
    #[serde(default)]
    pub suspended: bool,
    /// Captured Universal Execution State JSON when suspended at a checkpoint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ues: Option<Value>,
    /// Safepoint report JSON when suspended.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safepoint: Option<Value>,
}

/// Resume a checkpoint-produced UES on the same runtime (Continuum).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResumeCheckpointRequest {
    /// UES JSON document.
    pub ues: Value,
    /// Resource limits.
    pub limits: ExecutionLimits,
}

/// Response from same-runtime checkpoint resume.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResumeCheckpointResponse {
    /// Whether resume succeeded.
    pub success: bool,
    /// Captured stdout from post-checkpoint region.
    pub stdout: String,
    /// Captured stderr.
    pub stderr: String,
    /// Duration microseconds.
    pub duration_us: u64,
    /// Bindings after resume (PIR-tagged).
    #[serde(default)]
    pub bindings: Value,
    /// Optional exception.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exception: Option<Value>,
    /// Warnings (e.g. experimental).
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Restore request payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RestoreRequest {
    /// Bindings as PIR-tagged JSON object.
    pub bindings: Value,
    /// Resource limits.
    pub limits: ExecutionLimits,
}

/// Restore response payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RestoreResponse {
    /// Whether restore succeeded.
    pub success: bool,
    /// Warnings.
    #[serde(default)]
    pub warnings: Vec<String>,
    /// Restored binding summaries.
    #[serde(default)]
    pub restored: IndexMapCompat,
    /// Duration microseconds.
    pub duration_us: u64,
    /// Optional verification dump of restored values as PIR JSON.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bindings: Option<Value>,
}

/// Thin serde-friendly string map.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IndexMapCompat(pub std::collections::BTreeMap<String, String>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ndjson_round_trip() {
        let env = Envelope::request("ping", serde_json::json!({}));
        let line = env.to_ndjson_line().unwrap();
        let parsed = Envelope::from_ndjson_line(&line).unwrap();
        assert_eq!(parsed.op, "ping");
    }

    #[test]
    fn rejects_future_protocol_major() {
        let line = format!(
            r#"{{"v":{},"id":"00000000-0000-4000-8000-000000000001","op":"ping"}}"#,
            VERSION + 1
        );
        let err = Envelope::from_ndjson_line(&line).unwrap_err();
        assert_eq!(err.code, ErrorCode::ProtocolViolation);
    }

    #[test]
    fn hello_rejects_mismatched_protocol() {
        let resp = HelloResponse {
            protocol_version: VERSION + 7,
            runtime: RuntimeKind::Python,
            host_version: "3.12".into(),
            adapter_version: "0.1.0".into(),
        };
        let err = validate_hello(&resp).unwrap_err();
        assert_eq!(err.code, ErrorCode::ProtocolViolation);
    }
}
