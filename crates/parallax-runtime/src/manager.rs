//! Runtime manager coordinating adapters.

use crate::adapter::BoxRuntimeAdapter;
use crate::{CaptureOutcome, RuntimeHandle};
use parallax_core::{
    ErrorCode, ExecutionRequest, ExecutionResult, ExecutionState, ParallaxError, Remediation,
    RestoreResult, RuntimeKind, RuntimeStatus,
};
use parallax_ir::PirDocument;
use parallax_security::SandboxPolicy;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Central runtime registry and execution entrypoint.
pub struct RuntimeManager {
    adapters: RwLock<HashMap<String, BoxRuntimeAdapter>>,
    active: AtomicUsize,
    max_concurrent: usize,
}

impl RuntimeManager {
    /// Create an empty manager.
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            adapters: RwLock::new(HashMap::new()),
            active: AtomicUsize::new(0),
            max_concurrent: max_concurrent.max(1),
        }
    }

    /// Register an adapter (replaces existing for same kind).
    pub fn register(&self, adapter: BoxRuntimeAdapter) {
        let key = adapter.kind().as_str().to_string();
        self.adapters.write().insert(key, adapter);
    }

    /// List registered runtimes with live probe status.
    pub async fn list(&self) -> Vec<RuntimeHandle> {
        let adapters: Vec<BoxRuntimeAdapter> = self.adapters.read().values().cloned().collect();
        let mut out = Vec::new();
        for a in adapters {
            let status = a.probe().await;
            let mut metadata = a.metadata();
            if let RuntimeStatus::Ready = &status {
                // keep host_version if adapter filled it during probe via metadata — adapters set on construct
            }
            // Prefer status-derived version details left in metadata by adapters.
            let _ = &mut metadata;
            out.push(RuntimeHandle {
                kind: a.kind(),
                metadata: a.metadata(),
                capabilities: a.capabilities(),
                status,
            });
        }
        out.sort_by(|a, b| a.kind.as_str().cmp(b.kind.as_str()));
        out
    }

    /// Get adapter by kind.
    pub fn get(&self, kind: &RuntimeKind) -> parallax_core::Result<BoxRuntimeAdapter> {
        self.adapters
            .read()
            .get(kind.as_str())
            .cloned()
            .ok_or_else(|| {
                ParallaxError::new(
                    ErrorCode::RuntimeUnavailable,
                    format!("no adapter registered for {kind}"),
                )
                .with_runtime(kind.clone())
                .remediate(Remediation::new(
                    "Register the adapter or check installation",
                ))
            })
    }

    async fn with_slot<F, T>(&self, f: F) -> parallax_core::Result<T>
    where
        F: std::future::Future<Output = parallax_core::Result<T>>,
    {
        loop {
            let cur = self.active.load(Ordering::SeqCst);
            if cur >= self.max_concurrent {
                return Err(ParallaxError::new(
                    ErrorCode::ResourceLimitExceeded,
                    format!("bounded concurrency exceeded (max {})", self.max_concurrent),
                )
                .with_source("parallax-runtime")
                .with_operation("with_slot"));
            }
            if self
                .active
                .compare_exchange(cur, cur + 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                break;
            }
        }
        // Release the concurrency slot even if the future panics.
        struct ActiveGuard<'a>(&'a AtomicUsize);
        impl Drop for ActiveGuard<'_> {
            fn drop(&mut self) {
                self.0.fetch_sub(1, Ordering::SeqCst);
            }
        }
        let _guard = ActiveGuard(&self.active);
        f.await
    }

    /// Execute via the appropriate adapter.
    pub async fn execute(
        &self,
        request: ExecutionRequest,
        policy: &SandboxPolicy,
    ) -> parallax_core::Result<ExecutionResult> {
        let adapter = self.get(&request.runtime)?;
        parallax_core::check_adapter_interface(adapter.metadata().interface_version)?;
        adapter
            .capabilities()
            .require("execution", &request.runtime)?;
        if request.capture_state {
            adapter
                .capabilities()
                .require("globals", &request.runtime)?;
            adapter.capabilities().require("values", &request.runtime)?;
        }
        self.with_slot(adapter.execute(request, policy)).await
    }

    /// Execute and build PIR from captured bindings.
    pub async fn execute_and_capture(
        &self,
        mut request: ExecutionRequest,
        capture_names: &[String],
        policy: &SandboxPolicy,
    ) -> parallax_core::Result<CaptureOutcome> {
        request.capture_state = true;
        // Encode capture names into program if using File/Inline — adapters read request via protocol.
        // For CaptureBindings variant, names already present.
        use parallax_core::execution::ProgramSource;
        if !capture_names.is_empty() {
            match &request.program {
                ProgramSource::File { path } => {
                    let source = std::fs::read_to_string(path).map_err(|e| {
                        ParallaxError::new(ErrorCode::Io, e.to_string())
                            .with_operation("read_source")
                    })?;
                    request.program = ProgramSource::CaptureBindings {
                        source,
                        names: capture_names.to_vec(),
                        filename: path.clone(),
                    };
                }
                ProgramSource::Inline { source, filename } => {
                    request.program = ProgramSource::CaptureBindings {
                        source: source.clone(),
                        names: capture_names.to_vec(),
                        filename: filename.clone(),
                    };
                }
                ProgramSource::CaptureBindings { .. } => {}
                ProgramSource::Bytes { .. } => {}
            }
        }

        let execution = self.execute(request.clone(), policy).await?;
        let pir = if let Some(state) = &execution.state {
            // Prefer heap as bindings object if present.
            if state.heap.is_object() {
                PirDocument::from_bindings_json(&state.heap)?
            } else {
                PirDocument::new()
            }
        } else {
            PirDocument::new()
        };
        let state = execution.state.clone().unwrap_or_else(|| {
            ExecutionState::empty(
                request.runtime.clone(),
                DefaultCapabilities::for_kind(&request.runtime),
            )
        });
        Ok(CaptureOutcome {
            execution,
            pir,
            state,
        })
    }

    /// Restore bindings.
    pub async fn restore_bindings(
        &self,
        runtime: RuntimeKind,
        pir: &PirDocument,
        policy: &SandboxPolicy,
    ) -> parallax_core::Result<RestoreResult> {
        let adapter = self.get(&runtime)?;
        parallax_core::check_adapter_interface(adapter.metadata().interface_version)?;
        adapter.capabilities().require("globals", &runtime)?;
        adapter.capabilities().require("values", &runtime)?;
        pir.validate()?;
        self.with_slot(adapter.restore(pir, policy)).await
    }
}

struct DefaultCapabilities;
impl DefaultCapabilities {
    fn for_kind(kind: &RuntimeKind) -> parallax_core::RuntimeCapabilities {
        match kind {
            RuntimeKind::Python => parallax_core::RuntimeCapabilities::python(),
            RuntimeKind::JavaScript => parallax_core::RuntimeCapabilities::javascript(),
            RuntimeKind::Wasm => parallax_core::RuntimeCapabilities::wasm(),
            RuntimeKind::Other(_) => parallax_core::RuntimeCapabilities::none(),
        }
    }
}

impl Default for RuntimeManager {
    fn default() -> Self {
        Self::new(4)
    }
}
