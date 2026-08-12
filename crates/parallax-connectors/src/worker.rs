//! Experimental NDJSON worker-backed connectors (Ruby, PHP, Go, …).

use crate::catalog::{find, ConnectorDef, ConnectorMaturity};
use async_trait::async_trait;
use parallax_core::{
    CapabilityLevel, ErrorCode, ExceptionInfo, ExecutionRequest, ExecutionResult, ExecutionState,
    ParallaxError, ProgramSource, Remediation, RestoreResult, RuntimeCapabilities, RuntimeKind,
    RuntimeMetadata, RuntimeStatus, PARALLAX_VERSION,
};
use parallax_ir::PirDocument;
use parallax_protocol::{
    validate_hello, Envelope, ExecuteRequest, ExecuteResponse, HelloRequest, HelloResponse,
    ProtocolError, RestoreRequest, RestoreResponse,
};
use parallax_runtime::{BoxRuntimeAdapter, RuntimeAdapter, WorkerProcess};
use parallax_security::SandboxPolicy;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;

const RUBY_WORKER: &str = include_str!("../workers/worker.rb");
const PHP_WORKER: &str = include_str!("../workers/worker.php");
const GO_WORKER: &str = include_str!("../workers/worker.go");

/// How to invoke the host for a worker script.
#[derive(Clone)]
struct WorkerLaunch {
    program: PathBuf,
    args: Vec<String>,
    host_version: Option<String>,
}

/// Experimental connector backed by an embedded NDJSON worker.
pub struct WorkerConnector {
    def: &'static ConnectorDef,
    kind: RuntimeKind,
    launch: Option<WorkerLaunch>,
    caps: RuntimeCapabilities,
}

impl WorkerConnector {
    pub fn try_ruby() -> Option<BoxRuntimeAdapter> {
        let def = find("ruby")?;
        let host = find_on_path("ruby")?;
        let worker_path = materialize("ruby_worker.rb", RUBY_WORKER).ok()?;
        let ver = version_line(&host, &["--version"]);
        Some(Arc::new(Self {
            def,
            kind: RuntimeKind::Other("ruby".into()),
            launch: Some(WorkerLaunch {
                program: host,
                args: vec![worker_path.display().to_string()],
                host_version: ver,
            }),
            caps: scripting_caps(),
        }))
    }

    pub fn try_php() -> Option<BoxRuntimeAdapter> {
        let def = find("php")?;
        let host = find_on_path("php")?;
        let worker_path = materialize("php_worker.php", PHP_WORKER).ok()?;
        let ver = version_line(&host, &["--version"]);
        Some(Arc::new(Self {
            def,
            kind: RuntimeKind::Other("php".into()),
            launch: Some(WorkerLaunch {
                program: host,
                args: vec![worker_path.display().to_string()],
                host_version: ver,
            }),
            caps: scripting_caps(),
        }))
    }

    pub fn try_go() -> Option<BoxRuntimeAdapter> {
        let def = find("go")?;
        let host = find_on_path("go")?;
        let worker_path = materialize("go_worker.go", GO_WORKER).ok()?;
        let ver = version_line(&host, &["version"]);
        // Prefer `go run` of the worker (experimental; slower but no separate build step).
        Some(Arc::new(Self {
            def,
            kind: RuntimeKind::Other("go".into()),
            launch: Some(WorkerLaunch {
                program: host,
                args: vec!["run".into(), worker_path.display().to_string()],
                host_version: ver,
            }),
            caps: execute_only_caps(),
        }))
    }

    async fn spawn_ready(&self) -> parallax_core::Result<WorkerProcess> {
        let launch = self.launch.as_ref().ok_or_else(|| {
            ParallaxError::new(
                ErrorCode::RuntimeUnavailable,
                format!("{} host not found", self.def.id),
            )
            .with_runtime(self.kind.clone())
            .remediate(Remediation::new(format!(
                "Install {} and ensure it is on PATH",
                self.def.id
            )))
        })?;
        let mut worker =
            WorkerProcess::spawn(launch.program.clone(), &launch.args, self.kind.clone()).await?;
        let hello = Envelope::request(
            "hello",
            serde_json::to_value(HelloRequest {
                protocol_version: parallax_protocol::VERSION,
                runtime: self.kind.clone(),
            })?,
        );
        let resp = worker.request(hello, Duration::from_secs(30)).await?;
        if resp.ok != Some(true) {
            let err = resp.error.unwrap_or_else(|| ProtocolError {
                code: "RUNTIME_INITIALIZATION_FAILURE".into(),
                message: "hello failed".into(),
                diagnostic: None,
            });
            return Err(err.into_parallax(Some(self.kind.clone())));
        }
        let hello_payload: HelloResponse = serde_json::from_value(
            resp.payload.unwrap_or(serde_json::Value::Null),
        )
        .map_err(|e| {
            ParallaxError::new(
                ErrorCode::ProtocolViolation,
                format!("invalid hello response: {e}"),
            )
            .with_runtime(self.kind.clone())
        })?;
        validate_hello(&hello_payload)?;
        Ok(worker)
    }
}

fn scripting_caps() -> RuntimeCapabilities {
    let mut c = RuntimeCapabilities::none();
    c.values = CapabilityLevel::Yes;
    c.globals = CapabilityLevel::Partial;
    c.locals = CapabilityLevel::Partial;
    c.execution = CapabilityLevel::Yes;
    c.stdio_capture = CapabilityLevel::Yes;
    c.timeouts = CapabilityLevel::Yes;
    c.resource_limits = CapabilityLevel::Partial;
    c.cancellation = CapabilityLevel::Yes;
    c
}

fn execute_only_caps() -> RuntimeCapabilities {
    let mut c = RuntimeCapabilities::none();
    c.execution = CapabilityLevel::Experimental;
    c.stdio_capture = CapabilityLevel::Yes;
    c.timeouts = CapabilityLevel::Yes;
    c.cancellation = CapabilityLevel::Partial;
    c
}

fn materialize(name: &str, source: &str) -> parallax_core::Result<PathBuf> {
    let dir = std::env::temp_dir().join("parallax-workers");
    std::fs::create_dir_all(&dir).map_err(|e| {
        ParallaxError::new(ErrorCode::Io, e.to_string()).with_source("parallax-connectors")
    })?;
    let path = dir.join(name);
    std::fs::write(&path, source).map_err(|e| {
        ParallaxError::new(ErrorCode::Io, e.to_string()).with_source("parallax-connectors")
    })?;
    Ok(path)
}

fn find_on_path(bin: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(bin);
        if candidate.is_file() {
            return Some(candidate);
        }
        #[cfg(windows)]
        {
            for ext in ["exe", "cmd", "bat"] {
                let c = dir.join(format!("{bin}.{ext}"));
                if c.is_file() {
                    return Some(c);
                }
            }
        }
    }
    None
}

fn version_line(bin: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new(bin).args(args).output().ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    let line = s.lines().next().unwrap_or("").trim();
    if line.is_empty() {
        let e = String::from_utf8_lossy(&out.stderr);
        let line = e.lines().next().unwrap_or("").trim();
        if line.is_empty() {
            None
        } else {
            Some(line.to_string())
        }
    } else {
        Some(line.to_string())
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
        } => Ok((source.clone(), filename.clone(), names.clone())),
        ProgramSource::Bytes { .. } => Err(ParallaxError::new(
            ErrorCode::UnsupportedValue,
            "this connector does not execute raw bytes",
        )),
    }
}

#[async_trait]
impl RuntimeAdapter for WorkerConnector {
    fn kind(&self) -> RuntimeKind {
        self.kind.clone()
    }

    fn metadata(&self) -> RuntimeMetadata {
        let mut meta = RuntimeMetadata::builtin(
            self.kind.clone(),
            self.def.name,
            format!(
                "[{}] experimental NDJSON worker — {}",
                self.def.maturity.as_str(),
                self.def.notes
            ),
        );
        meta.adapter_version = PARALLAX_VERSION.to_string();
        if let Some(l) = &self.launch {
            meta.host_version = l.host_version.clone();
        }
        meta
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        self.caps.clone()
    }

    async fn probe(&self) -> RuntimeStatus {
        if self.launch.is_some() {
            RuntimeStatus::Ready
        } else {
            RuntimeStatus::Unavailable {
                reason: format!("{} host not found on PATH", self.def.id),
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
        let limits = policy.limits.clone();
        let max_message_bytes = limits.max_message_bytes;
        let wait = limits.timeout + Duration::from_secs(60);
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
                continuum: false,
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
            return Err(err.into_parallax(Some(self.kind.clone())));
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
            let mut st = ExecutionState::empty(self.kind.clone(), self.caps.clone());
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
                .unwrap_or("Error")
                .to_string(),
            message: v
                .get("message")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            stack: None,
        });

        Ok(ExecutionResult {
            execution_id,
            runtime: self.kind.clone(),
            success: payload.success,
            value: None,
            stdout: payload.stdout,
            stderr: payload.stderr,
            duration_us: payload.duration_us,
            exception,
            state,
            suspended: payload.suspended,
            ues: None,
            safepoint: None,
            stats: Default::default(),
        })
    }

    async fn restore(
        &self,
        pir: &PirDocument,
        policy: &SandboxPolicy,
    ) -> parallax_core::Result<RestoreResult> {
        if self.caps.values == CapabilityLevel::No {
            return Err(ParallaxError::new(
                ErrorCode::UnsupportedValue,
                format!("{} does not support binding restore", self.def.id),
            )
            .with_runtime(self.kind.clone())
            .remediate(Remediation::new(
                "Use python/javascript for value migration, or contribute restore for this connector",
            )));
        }
        let limits = policy.limits.clone();
        let max_message_bytes = limits.max_message_bytes;
        let wait = limits.timeout + Duration::from_secs(30);
        let mut worker = self.spawn_ready().await?;
        let env = Envelope::request(
            "restore",
            serde_json::to_value(RestoreRequest {
                bindings: pir.bindings_to_json(),
                limits,
            })?,
        );
        let resp = worker.request_bounded(env, wait, max_message_bytes).await;
        worker.shutdown().await;
        let resp = resp?;
        if resp.ok != Some(true) {
            let err = resp.error.unwrap_or_else(|| ProtocolError {
                code: "UNSUPPORTED_VALUE".into(),
                message: "restore failed".into(),
                diagnostic: None,
            });
            return Err(err.into_parallax(Some(self.kind.clone())));
        }
        let payload: RestoreResponse =
            serde_json::from_value(resp.payload.unwrap_or(serde_json::Value::Null))?;
        Ok(RestoreResult {
            success: payload.success,
            runtime: self.kind.clone(),
            warnings: payload.warnings,
            restored_bindings: payload.restored.0.into_iter().collect(),
            duration_us: payload.duration_us,
        })
    }
}

/// Register experimental worker connectors when host tools exist.
/// Returns ids that were registered so scaffolds can skip them.
pub fn register_experimental_workers(
    manager: &parallax_runtime::RuntimeManager,
) -> Vec<&'static str> {
    let mut registered = Vec::new();
    for (id, factory) in [
        (
            "ruby",
            WorkerConnector::try_ruby as fn() -> Option<BoxRuntimeAdapter>,
        ),
        ("php", WorkerConnector::try_php),
        ("go", WorkerConnector::try_go),
    ] {
        match factory() {
            Some(adapter) => {
                debug!(id, "registered experimental worker connector");
                manager.register(adapter);
                registered.push(id);
            }
            None => {
                debug!(
                    id,
                    "experimental worker host unavailable — scaffold will register instead"
                );
            }
        }
    }
    let _ = ConnectorMaturity::Experimental;
    registered
}
