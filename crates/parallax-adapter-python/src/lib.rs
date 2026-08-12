//! Python runtime adapter (subprocess worker).

#![deny(unsafe_code)]
#![warn(missing_docs)]

use async_trait::async_trait;
use parallax_core::{
    ErrorCode, ExceptionInfo, ExecutionRequest, ExecutionResult, ExecutionState, ParallaxError,
    ProgramSource, Remediation, RestoreResult, RuntimeCapabilities, RuntimeKind, RuntimeMetadata,
    RuntimeStatus, PARALLAX_VERSION,
};
use parallax_ir::PirDocument;
use parallax_protocol::{
    validate_hello, Envelope, ExecuteRequest, ExecuteResponse, HelloRequest, HelloResponse,
    ProtocolError, RestoreRequest, RestoreResponse, ResumeCheckpointRequest,
    ResumeCheckpointResponse,
};
use parallax_runtime::{discover_python, BoxRuntimeAdapter, RuntimeAdapter, WorkerProcess};
use parallax_security::SandboxPolicy;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};

const WORKER_SOURCE: &str = include_str!("../worker.py");

/// Python adapter.
pub struct PythonAdapter {
    /// Discovered interpreter path, if any.
    pub python: Option<PathBuf>,
    host_version: Option<String>,
    worker_path: PathBuf,
}

impl PythonAdapter {
    /// Create adapter, discovering Python and materializing the worker script.
    pub fn new() -> parallax_core::Result<Self> {
        let discovered = discover_python();
        let worker_path = materialize_worker()?;
        Ok(Self {
            python: discovered.as_ref().map(|d| d.path.clone()),
            host_version: discovered.and_then(|d| d.version),
            worker_path,
        })
    }

    /// Boxed factory helper.
    pub fn boxed() -> parallax_core::Result<BoxRuntimeAdapter> {
        Ok(Arc::new(Self::new()?))
    }

    async fn spawn_ready(&self) -> parallax_core::Result<WorkerProcess> {
        let python = self.python.as_ref().ok_or_else(|| {
            ParallaxError::new(
                ErrorCode::RuntimeUnavailable,
                "Python interpreter not found on this host",
            )
            .with_runtime(RuntimeKind::Python)
            .with_source("parallax-adapter-python")
            .remediate(Remediation::with_detail(
                "Install Python 3 and ensure `python`, `python3`, or `py` works",
                "On Windows, disable the Microsoft Store python alias or install from python.org",
            ))
        })?;

        let args = vec![self.worker_path.to_string_lossy().to_string()];
        let mut worker = WorkerProcess::spawn(python.clone(), &args, RuntimeKind::Python).await?;
        let hello = Envelope::request(
            "hello",
            serde_json::to_value(HelloRequest {
                protocol_version: parallax_protocol::VERSION,
                runtime: RuntimeKind::Python,
            })?,
        );
        let resp = worker.request(hello, Duration::from_secs(10)).await?;
        if resp.ok != Some(true) {
            let err = resp.error.unwrap_or_else(|| ProtocolError {
                code: "RUNTIME_INITIALIZATION_FAILURE".into(),
                message: "hello failed".into(),
                diagnostic: None,
            });
            return Err(err.into_parallax(Some(RuntimeKind::Python)));
        }
        let hello_payload: HelloResponse = serde_json::from_value(
            resp.payload.unwrap_or(serde_json::Value::Null),
        )
        .map_err(|e| {
            ParallaxError::new(
                ErrorCode::ProtocolViolation,
                format!("invalid hello response: {e}"),
            )
            .with_runtime(RuntimeKind::Python)
            .with_source("parallax-adapter-python")
        })?;
        validate_hello(&hello_payload)?;
        Ok(worker)
    }
}

fn materialize_worker() -> parallax_core::Result<PathBuf> {
    let dir = std::env::temp_dir().join("parallax-workers");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("python_worker.py");
    std::fs::write(&path, WORKER_SOURCE)?;
    Ok(path)
}

fn validate_capture_name(name: &str) -> parallax_core::Result<()> {
    let ok = name
        .chars()
        .next()
        .map(|c| c.is_ascii_alphabetic() || c == '_')
        .unwrap_or(false)
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if ok {
        Ok(())
    } else {
        Err(ParallaxError::new(
            ErrorCode::InvalidArgument,
            format!("invalid capture binding name: {name}"),
        )
        .with_runtime(RuntimeKind::Python)
        .with_operation("validate_capture_name")
        .remediate(Remediation::new(
            "Use ASCII identifier names (letters, digits, underscore)",
        )))
    }
}

fn load_source(program: &ProgramSource) -> parallax_core::Result<(String, String, Vec<String>)> {
    match program {
        ProgramSource::File { path } => {
            let source = std::fs::read_to_string(path)?;
            Ok((source, path.clone(), Vec::new()))
        }
        ProgramSource::Inline { source, filename } => {
            Ok((source.clone(), filename.clone(), Vec::new()))
        }
        ProgramSource::CaptureBindings {
            source,
            names,
            filename,
        } => {
            for name in names {
                validate_capture_name(name)?;
            }
            Ok((source.clone(), filename.clone(), names.clone()))
        }
        ProgramSource::Bytes { .. } => Err(ParallaxError::new(
            ErrorCode::UnsupportedValue,
            "Python adapter does not execute raw bytes",
        )
        .with_runtime(RuntimeKind::Python)),
    }
}

#[async_trait]
impl RuntimeAdapter for PythonAdapter {
    fn kind(&self) -> RuntimeKind {
        RuntimeKind::Python
    }

    fn metadata(&self) -> RuntimeMetadata {
        let mut meta = RuntimeMetadata::builtin(
            RuntimeKind::Python,
            "CPython",
            "Python adapter via NDJSON subprocess worker",
        );
        meta.host_version = self.host_version.clone();
        meta.adapter_version = PARALLAX_VERSION.to_string();
        meta
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        RuntimeCapabilities::python()
    }

    async fn probe(&self) -> RuntimeStatus {
        if self.python.is_some() {
            RuntimeStatus::Ready
        } else {
            RuntimeStatus::Unavailable {
                reason: "Python not found (tried python, python3, py, and common install paths)"
                    .into(),
            }
        }
    }

    async fn execute(
        &self,
        request: ExecutionRequest,
        policy: &SandboxPolicy,
    ) -> parallax_core::Result<ExecutionResult> {
        let (source, filename, mut capture) = load_source(&request.program)?;
        if request.capture_state && capture.is_empty() {
            capture.push("state".into());
        }
        for name in &capture {
            validate_capture_name(name)?;
        }
        let limits = policy.limits.clone();
        let max_message_bytes = limits.max_message_bytes;
        let wait = limits.timeout + Duration::from_secs(5);
        let execution_id = request.execution_id;
        let program_label = filename.clone();

        let mut worker = self.spawn_ready().await?;
        let env = Envelope::request(
            "execute",
            serde_json::to_value(ExecuteRequest {
                source,
                filename,
                capture,
                limits,
                continuum: request.continuum,
            })?,
        );
        let resp = worker.request_bounded(env, wait, max_message_bytes).await;
        worker.shutdown().await;
        let resp = resp?;
        if resp.ok != Some(true) {
            let err = resp.error.unwrap_or_else(|| ProtocolError {
                code: "EXECUTION_FAILURE".into(),
                message: "execute failed".into(),
                diagnostic: None,
            });
            return Err(err.into_parallax(Some(RuntimeKind::Python)));
        }
        let payload: ExecuteResponse =
            serde_json::from_value(resp.payload.unwrap_or(serde_json::Value::Null))?;

        let state = if payload
            .bindings
            .as_object()
            .map(|o| !o.is_empty())
            .unwrap_or(false)
        {
            let pir = PirDocument::from_bindings_json(&payload.bindings)?;
            let mut st = ExecutionState::empty(RuntimeKind::Python, RuntimeCapabilities::python());
            st.heap = pir.bindings_to_json();
            st.program = Some(program_label);
            Some(st)
        } else {
            None
        };

        let exception = payload.exception.as_ref().map(|v| ExceptionInfo {
            type_name: v
                .get("type_name")
                .and_then(|x| x.as_str())
                .unwrap_or("Exception")
                .into(),
            message: v
                .get("message")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .into(),
            stack: v
                .get("stack")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string()),
        });

        Ok(ExecutionResult {
            execution_id,
            runtime: RuntimeKind::Python,
            success: payload.success || payload.suspended,
            value: None,
            stdout: payload.stdout,
            stderr: payload.stderr,
            duration_us: payload.duration_us,
            exception,
            state,
            suspended: payload.suspended,
            ues: payload.ues,
            safepoint: payload.safepoint,
            stats: Default::default(),
        })
    }

    async fn restore(
        &self,
        pir: &PirDocument,
        policy: &SandboxPolicy,
    ) -> parallax_core::Result<RestoreResult> {
        let bindings = pir.bindings_to_json();
        let limits = policy.limits.clone();
        let max_message_bytes = limits.max_message_bytes;
        let wait = limits.timeout + Duration::from_secs(5);
        let mut worker = self.spawn_ready().await?;
        let env = Envelope::request(
            "restore",
            serde_json::to_value(RestoreRequest { bindings, limits })?,
        );
        let resp = worker.request_bounded(env, wait, max_message_bytes).await;
        worker.shutdown().await;
        let resp = resp?;
        if resp.ok != Some(true) {
            let err = resp.error.unwrap_or_else(|| ProtocolError {
                code: "RESTORE_FAILURE".into(),
                message: "restore failed".into(),
                diagnostic: None,
            });
            return Err(err.into_parallax(Some(RuntimeKind::Python)));
        }
        let payload: RestoreResponse =
            serde_json::from_value(resp.payload.unwrap_or(serde_json::Value::Null))?;
        Ok(RestoreResult {
            success: payload.success,
            runtime: RuntimeKind::Python,
            warnings: payload.warnings,
            restored_bindings: payload.restored.0.into_iter().collect(),
            duration_us: payload.duration_us,
        })
    }
}

impl PythonAdapter {
    /// Same-runtime resume from a checkpoint-produced UES (Continuum experimental).
    pub async fn resume_checkpoint(
        &self,
        ues: serde_json::Value,
        policy: &SandboxPolicy,
    ) -> parallax_core::Result<ResumeCheckpointResponse> {
        let limits = policy.limits.clone();
        let max_message_bytes = limits.max_message_bytes;
        let wait = limits.timeout + Duration::from_secs(5);
        let mut worker = self.spawn_ready().await?;
        let env = Envelope::request(
            "resume_checkpoint",
            serde_json::to_value(ResumeCheckpointRequest { ues, limits })?,
        );
        let resp = worker.request_bounded(env, wait, max_message_bytes).await;
        worker.shutdown().await;
        let resp = resp?;
        if resp.ok != Some(true) {
            let err = resp.error.unwrap_or_else(|| ProtocolError {
                code: "RESTORE_FAILURE".into(),
                message: "resume_checkpoint failed".into(),
                diagnostic: None,
            });
            return Err(err.into_parallax(Some(RuntimeKind::Python)));
        }
        serde_json::from_value(resp.payload.unwrap_or(serde_json::Value::Null)).map_err(|e| {
            ParallaxError::new(
                ErrorCode::ProtocolViolation,
                format!("invalid resume_checkpoint response: {e}"),
            )
            .with_runtime(RuntimeKind::Python)
            .with_source("parallax-adapter-python")
        })
    }
}

/// Always register; probe reports unavailable if Python missing.
pub fn register_lenient(manager: &parallax_runtime::RuntimeManager) {
    match PythonAdapter::new() {
        Ok(adapter) => {
            debug!(python = ?adapter.python, "registered python adapter");
            manager.register(Arc::new(adapter));
        }
        Err(e) => warn!("could not materialize python worker: {e}"),
    }
}
