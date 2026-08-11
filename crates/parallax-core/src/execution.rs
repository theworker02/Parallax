//! Execution requests, results, and language-neutral execution state.

use crate::capabilities::RuntimeCapabilities;
use crate::ids::{ExecutionId, ObjectId};
use crate::runtime::RuntimeKind;
use crate::semantic::ConversionPolicy;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Limits applied to a single execution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionLimits {
    /// Wall-clock timeout.
    #[serde(with = "duration_millis")]
    pub timeout: Duration,
    /// Maximum captured stdout+stderr bytes.
    pub max_output_bytes: u64,
    /// Maximum protocol message size.
    pub max_message_bytes: u64,
    /// Soft memory hint in bytes (enforced where supported).
    pub max_memory_bytes: Option<u64>,
    /// Maximum WASM fuel / instruction budget when applicable.
    pub max_fuel: Option<u64>,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_output_bytes: 1_048_576,
            max_message_bytes: 16_777_216,
            max_memory_bytes: Some(256 * 1024 * 1024),
            max_fuel: Some(10_000_000),
        }
    }
}

mod duration_millis {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(d: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let ms = u64::try_from(d.as_millis()).unwrap_or(u64::MAX);
        serializer.serialize_u64(ms)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ms = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(ms))
    }
}

/// What to execute.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProgramSource {
    /// Source file on disk.
    File {
        /// Absolute or relative path.
        path: String,
    },
    /// Inline source text.
    Inline {
        /// Source code.
        source: String,
        /// Filename hint for diagnostics.
        filename: String,
    },
    /// Precompiled bytes (e.g. WASM module).
    Bytes {
        /// Raw module bytes (base64 in JSON).
        #[serde(with = "serde_bytes_b64")]
        data: Vec<u8>,
        /// Filename hint.
        filename: String,
    },
    /// Capture named bindings after evaluating a prelude snippet.
    CaptureBindings {
        /// Source that defines bindings.
        source: String,
        /// Binding names to capture into PIR.
        names: Vec<String>,
        /// Filename hint.
        filename: String,
    },
}

mod serde_bytes_b64 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::Error;
        let encoded = base64_encode(bytes).map_err(S::Error::custom)?;
        serializer.serialize_str(&encoded)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let s = String::deserialize(deserializer)?;
        base64_decode(&s).map_err(D::Error::custom)
    }

    fn base64_encode(bytes: &[u8]) -> Result<String, String> {
        // Minimal base64 without extra dependency in core.
        const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for chunk in bytes.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
            let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
            let n = (b0 << 16) | (b1 << 8) | b2;
            out.push(TABLE[((n >> 18) & 63) as usize] as char);
            out.push(TABLE[((n >> 12) & 63) as usize] as char);
            out.push(if chunk.len() > 1 {
                TABLE[((n >> 6) & 63) as usize] as char
            } else {
                '='
            });
            out.push(if chunk.len() > 2 {
                TABLE[(n & 63) as usize] as char
            } else {
                '='
            });
        }
        Ok(out)
    }

    fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
        fn val(c: u8) -> Result<u8, String> {
            match c {
                b'A'..=b'Z' => Ok(c - b'A'),
                b'a'..=b'z' => Ok(c - b'a' + 26),
                b'0'..=b'9' => Ok(c - b'0' + 52),
                b'+' => Ok(62),
                b'/' => Ok(63),
                _ => Err(format!("invalid base64 byte: {}", c)),
            }
        }
        let bytes = input.as_bytes();
        if bytes.len() % 4 != 0 {
            return Err("invalid base64 length".into());
        }
        let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
        for chunk in bytes.chunks(4) {
            let n = ((val(chunk[0])? as u32) << 18)
                | ((val(chunk[1])? as u32) << 12)
                | ((if chunk[2] == b'=' {
                    0
                } else {
                    val(chunk[2])? as u32
                }) << 6)
                | (if chunk[3] == b'=' {
                    0
                } else {
                    val(chunk[3])? as u32
                });
            out.push(((n >> 16) & 0xff) as u8);
            if chunk[2] != b'=' {
                out.push(((n >> 8) & 0xff) as u8);
            }
            if chunk[3] != b'=' {
                out.push((n & 0xff) as u8);
            }
        }
        Ok(out)
    }
}

/// Request to execute guest code.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionRequest {
    /// Correlation id.
    pub execution_id: ExecutionId,
    /// Target runtime.
    pub runtime: RuntimeKind,
    /// Program to run.
    pub program: ProgramSource,
    /// Resource limits.
    pub limits: ExecutionLimits,
    /// Optional entry function for WASM / callable targets.
    pub entry: Option<String>,
    /// Optional arguments encoded as JSON for simple call conventions.
    pub args: Vec<serde_json::Value>,
    /// Whether to capture state after execution.
    pub capture_state: bool,
    /// Enable Continuum explicit-checkpoint capture (`parallax.checkpoint`).
    #[serde(default)]
    pub continuum: bool,
}

impl ExecutionRequest {
    /// Execute a file with default limits.
    pub fn file(runtime: RuntimeKind, path: impl Into<String>) -> Self {
        Self {
            execution_id: ExecutionId::new(),
            runtime,
            program: ProgramSource::File { path: path.into() },
            limits: ExecutionLimits::default(),
            entry: None,
            args: Vec::new(),
            capture_state: false,
            continuum: false,
        }
    }
}

/// Outcome of an execution attempt.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Correlation id.
    pub execution_id: ExecutionId,
    /// Runtime that executed.
    pub runtime: RuntimeKind,
    /// Whether the guest completed successfully.
    pub success: bool,
    /// Guest return value as JSON when applicable.
    pub value: Option<serde_json::Value>,
    /// Captured stdout.
    pub stdout: String,
    /// Captured stderr.
    pub stderr: String,
    /// Wall time in microseconds.
    pub duration_us: u64,
    /// Optional exception summary.
    pub exception: Option<ExceptionInfo>,
    /// Optional captured execution state (PIR-backed).
    pub state: Option<ExecutionState>,
    /// True when paused at an explicit Continuum safepoint.
    #[serde(default)]
    pub suspended: bool,
    /// Captured Universal Execution State JSON (Continuum), when suspended.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ues: Option<serde_json::Value>,
    /// Safepoint report JSON when suspended.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safepoint: Option<serde_json::Value>,
    /// Adapter-specific stats.
    pub stats: IndexMap<String, serde_json::Value>,
}

/// Exception / trap information.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExceptionInfo {
    /// Exception type name.
    pub type_name: String,
    /// Message.
    pub message: String,
    /// Optional stack / traceback.
    pub stack: Option<String>,
}

/// Language-neutral execution state.
///
/// Not every runtime populates every field. Consult `capabilities`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionState {
    /// Schema version for this state blob.
    pub version: u32,
    /// Origin runtime.
    pub runtime: RuntimeKind,
    /// Declared capabilities at capture time.
    pub capabilities: RuntimeCapabilities,
    /// Program identity / source reference.
    pub program: Option<String>,
    /// Instruction / bytecode position when known.
    pub instruction_position: Option<String>,
    /// Stack frames (experimental; often empty).
    pub stack_frames: Vec<StackFrame>,
    /// Global bindings as a PIR object graph reference root.
    pub globals_root: Option<ObjectId>,
    /// Local bindings root.
    pub locals_root: Option<ObjectId>,
    /// Heap / object graph encoded as PIR document bytes or embedded JSON.
    /// Concrete PIR lives in `parallax-ir`; here we store a serialized document.
    pub heap: serde_json::Value,
    /// Pending exception if any.
    pub pending_exception: Option<ExceptionInfo>,
    /// Async state placeholder.
    pub async_state: Option<serde_json::Value>,
    /// Capability tokens granted to the execution.
    pub granted_capabilities: Vec<String>,
    /// Free-form runtime metadata.
    pub runtime_metadata: IndexMap<String, serde_json::Value>,
    /// Migration metadata.
    pub migration_metadata: IndexMap<String, serde_json::Value>,
    /// Conversion policy used when producing this state.
    pub conversion_policy: ConversionPolicy,
}

impl ExecutionState {
    /// Create an empty state shell for a runtime.
    pub fn empty(runtime: RuntimeKind, capabilities: RuntimeCapabilities) -> Self {
        Self {
            version: crate::PIR_SCHEMA_VERSION,
            runtime,
            capabilities,
            program: None,
            instruction_position: None,
            stack_frames: Vec::new(),
            globals_root: None,
            locals_root: None,
            heap: serde_json::json!({}),
            pending_exception: None,
            async_state: None,
            granted_capabilities: Vec::new(),
            runtime_metadata: IndexMap::new(),
            migration_metadata: IndexMap::new(),
            conversion_policy: ConversionPolicy::default(),
        }
    }
}

/// A single stack frame when capture is available.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StackFrame {
    /// Function name if known.
    pub function: Option<String>,
    /// Source location.
    pub location: Option<String>,
    /// Locals object id.
    pub locals: Option<ObjectId>,
}

/// Result of restoring state into a runtime.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RestoreResult {
    /// Whether restore succeeded.
    pub success: bool,
    /// Runtime that received the state.
    pub runtime: RuntimeKind,
    /// Warnings emitted during restore.
    pub warnings: Vec<String>,
    /// Bindings restored (name → summary).
    pub restored_bindings: IndexMap<String, String>,
    /// Duration in microseconds.
    pub duration_us: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_limits_are_bounded() {
        let limits = ExecutionLimits::default();
        assert!(limits.timeout.as_secs() > 0);
        assert!(limits.max_output_bytes > 0);
    }
}
