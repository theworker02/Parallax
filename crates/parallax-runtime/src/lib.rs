//! Runtime manager, adapter trait, and worker orchestration.

#![deny(unsafe_code)]
#![warn(missing_docs)]

mod adapter;
mod discover;
mod manager;
mod worker;

pub use adapter::{AdapterFactory, BoxRuntimeAdapter, RuntimeAdapter};
pub use discover::{discover_javascript, discover_python, DiscoveredBinary};
pub use manager::RuntimeManager;
pub use worker::WorkerProcess;

use parallax_core::{
    ExecutionRequest, ExecutionResult, ExecutionState, RestoreResult, RuntimeCapabilities,
    RuntimeKind, RuntimeMetadata, RuntimeStatus,
};
use parallax_ir::PirDocument;
use parallax_security::SandboxPolicy;
use std::sync::Arc;

/// Handle describing a registered runtime.
#[derive(Clone, Debug)]
pub struct RuntimeHandle {
    /// Kind.
    pub kind: RuntimeKind,
    /// Metadata.
    pub metadata: RuntimeMetadata,
    /// Capabilities.
    pub capabilities: RuntimeCapabilities,
    /// Availability.
    pub status: RuntimeStatus,
}

/// Shared manager type.
pub type SharedManager = Arc<RuntimeManager>;

/// Capture bindings helper result.
#[derive(Clone, Debug)]
pub struct CaptureOutcome {
    /// Execution result.
    pub execution: ExecutionResult,
    /// PIR document built from captured bindings.
    pub pir: PirDocument,
    /// Execution state shell.
    pub state: ExecutionState,
}

/// High-level execute+capture API used by CLI migrate/snapshot.
pub async fn execute_and_capture(
    manager: &RuntimeManager,
    request: ExecutionRequest,
    capture_names: &[String],
    policy: &SandboxPolicy,
) -> parallax_core::Result<CaptureOutcome> {
    manager
        .execute_and_capture(request, capture_names, policy)
        .await
}

/// Restore PIR bindings into a runtime.
pub async fn restore_bindings(
    manager: &RuntimeManager,
    runtime: RuntimeKind,
    pir: &PirDocument,
    policy: &SandboxPolicy,
) -> parallax_core::Result<RestoreResult> {
    manager.restore_bindings(runtime, pir, policy).await
}
