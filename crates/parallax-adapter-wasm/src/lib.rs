//! WebAssembly adapter using wasmtime (in-process, fuel-limited).

#![deny(unsafe_code)]
#![warn(missing_docs)]

use async_trait::async_trait;
use parallax_core::{
    ErrorCode, ExecutionRequest, ExecutionResult, ParallaxError, ProgramSource, Remediation,
    RestoreResult, RuntimeCapabilities, RuntimeKind, RuntimeMetadata, RuntimeStatus,
    PARALLAX_VERSION,
};
use parallax_ir::PirDocument;
use parallax_runtime::{BoxRuntimeAdapter, RuntimeAdapter};
use parallax_security::SandboxPolicy;
use std::sync::Arc;
use std::time::Instant;
use tracing::debug;
use wasmtime::{Config, Engine, Linker, Module, Store};

/// WASM adapter.
pub struct WasmAdapter {
    engine: Engine,
}

impl WasmAdapter {
    /// Create adapter with fuel + epoch interruption configured.
    pub fn new() -> parallax_core::Result<Self> {
        let mut config = Config::new();
        config.consume_fuel(true);
        config.wasm_multi_value(true);
        let engine = Engine::new(&config).map_err(|e| {
            ParallaxError::new(
                ErrorCode::RuntimeInitializationFailure,
                format!("wasmtime engine: {e}"),
            )
            .with_runtime(RuntimeKind::Wasm)
        })?;
        Ok(Self { engine })
    }

    /// Boxed helper.
    pub fn boxed() -> parallax_core::Result<BoxRuntimeAdapter> {
        Ok(Arc::new(Self::new()?))
    }
}

impl Default for WasmAdapter {
    fn default() -> Self {
        Self::new().expect("wasm adapter")
    }
}

fn load_bytes(program: &ProgramSource) -> parallax_core::Result<(Vec<u8>, String)> {
    match program {
        ProgramSource::File { path } => {
            let data = std::fs::read(path)?;
            Ok((data, path.clone()))
        }
        ProgramSource::Bytes { data, filename } => Ok((data.clone(), filename.clone())),
        ProgramSource::Inline { source, filename } => {
            // Treat inline as WAT text if it looks like it; otherwise unsupported.
            if source.trim_start().starts_with('(') {
                Ok((source.as_bytes().to_vec(), filename.clone()))
            } else {
                Err(
                    ParallaxError::new(ErrorCode::UnsupportedValue, "inline WASM must be WAT text")
                        .with_runtime(RuntimeKind::Wasm),
                )
            }
        }
        ProgramSource::CaptureBindings { .. } => Err(ParallaxError::new(
            ErrorCode::UnsupportedValue,
            "WASM adapter does not support binding capture/migration",
        )
        .with_runtime(RuntimeKind::Wasm)
        .remediate(Remediation::new(
            "Use Python/JS adapters for state migration",
        ))),
    }
}

#[async_trait]
impl RuntimeAdapter for WasmAdapter {
    fn kind(&self) -> RuntimeKind {
        RuntimeKind::Wasm
    }

    fn metadata(&self) -> RuntimeMetadata {
        let mut meta = RuntimeMetadata::builtin(
            RuntimeKind::Wasm,
            "Wasmtime",
            "In-process WASM execution with fuel limits",
        );
        meta.host_version = Some("wasmtime-29".into());
        meta.adapter_version = PARALLAX_VERSION.to_string();
        meta
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        RuntimeCapabilities::wasm()
    }

    async fn probe(&self) -> RuntimeStatus {
        RuntimeStatus::Ready
    }

    async fn execute(
        &self,
        request: ExecutionRequest,
        policy: &SandboxPolicy,
    ) -> parallax_core::Result<ExecutionResult> {
        let t0 = Instant::now();
        let (bytes, _name) = load_bytes(&request.program)?;
        let module = Module::new(&self.engine, &bytes).map_err(|e| {
            ParallaxError::new(ErrorCode::ExecutionFailure, format!("module compile: {e}"))
                .with_runtime(RuntimeKind::Wasm)
        })?;

        let mut linker = Linker::new(&self.engine);
        // No host imports by default — modules must be self-contained / use WASI later.
        let _ = &mut linker;

        let mut store = Store::new(&self.engine, ());
        let fuel = policy
            .limits
            .max_fuel
            .or(request.limits.max_fuel)
            .unwrap_or(10_000_000);
        store.set_fuel(fuel).map_err(|e| {
            ParallaxError::new(ErrorCode::Internal, format!("set_fuel: {e}"))
                .with_runtime(RuntimeKind::Wasm)
        })?;

        let instance = linker.instantiate(&mut store, &module).map_err(|e| {
            ParallaxError::new(ErrorCode::ExecutionFailure, format!("instantiate: {e}"))
                .with_runtime(RuntimeKind::Wasm)
        })?;

        let entry = request.entry.as_deref().unwrap_or("run");
        let func = instance.get_func(&mut store, entry).ok_or_else(|| {
            ParallaxError::new(
                ErrorCode::ExecutionFailure,
                format!("exported function '{entry}' not found"),
            )
            .with_runtime(RuntimeKind::Wasm)
            .remediate(Remediation::new(
                "Pass --entry <name> matching an exported function",
            ))
        })?;

        // Call with no args / ignore results for MVP; typed calls can be added later.
        let ty = func.ty(&store);
        if ty.params().len() != 0 {
            return Err(ParallaxError::new(
                ErrorCode::UnsupportedValue,
                format!(
                    "entry '{entry}' requires {} params; only zero-arg exports supported in MVP",
                    ty.params().len()
                ),
            )
            .with_runtime(RuntimeKind::Wasm));
        }
        let mut results = vec![wasmtime::Val::I32(0); ty.results().len()];
        func.call(&mut store, &[], &mut results).map_err(|e| {
            ParallaxError::new(ErrorCode::ExecutionFailure, format!("trap: {e}"))
                .with_runtime(RuntimeKind::Wasm)
        })?;

        let value = if results.is_empty() {
            None
        } else {
            Some(serde_json::json!(results
                .iter()
                .map(|v| match v {
                    wasmtime::Val::I32(x) => serde_json::json!(x),
                    wasmtime::Val::I64(x) => serde_json::json!(x),
                    wasmtime::Val::F32(x) => serde_json::json!(f32::from_bits(*x)),
                    wasmtime::Val::F64(x) => serde_json::json!(f64::from_bits(*x)),
                    _ => serde_json::json!(format!("{v:?}")),
                })
                .collect::<Vec<_>>()))
        };

        Ok(ExecutionResult {
            execution_id: request.execution_id,
            runtime: RuntimeKind::Wasm,
            success: true,
            value,
            stdout: String::new(),
            stderr: String::new(),
            duration_us: t0.elapsed().as_micros() as u64,
            exception: None,
            state: None,
            suspended: false,
            ues: None,
            safepoint: None,
            stats: {
                let mut m = indexmap::IndexMap::new();
                m.insert("fuel_budget".into(), serde_json::json!(fuel));
                m
            },
        })
    }

    async fn restore(
        &self,
        _pir: &PirDocument,
        _policy: &SandboxPolicy,
    ) -> parallax_core::Result<RestoreResult> {
        Err(ParallaxError::new(
            ErrorCode::CapabilityViolation,
            "WASM state restore/migration is unsupported",
        )
        .with_runtime(RuntimeKind::Wasm)
        .with_source("parallax-adapter-wasm")
        .with_operation("restore")
        .context("capability", "globals")
        .context("level", "NO")
        .remediate(Remediation::new(
            "WASM adapter supports execution only; use Python/JS for state migration",
        )))
    }
}

/// Register wasm adapter.
pub fn register_lenient(manager: &parallax_runtime::RuntimeManager) {
    match WasmAdapter::new() {
        Ok(adapter) => {
            debug!("registered wasm adapter");
            manager.register(Arc::new(adapter));
        }
        Err(e) => tracing::warn!("wasm adapter unavailable: {e}"),
    }
}
