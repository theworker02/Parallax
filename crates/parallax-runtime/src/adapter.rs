//! Runtime adapter trait.

use async_trait::async_trait;
use parallax_core::{
    ExecutionRequest, ExecutionResult, RestoreResult, RuntimeCapabilities, RuntimeKind,
    RuntimeMetadata, RuntimeStatus,
};
use parallax_ir::PirDocument;
use parallax_security::SandboxPolicy;
use std::sync::Arc;

/// Trait implemented by language adapters.
#[async_trait]
pub trait RuntimeAdapter: Send + Sync {
    /// Runtime kind.
    fn kind(&self) -> RuntimeKind;

    /// Static metadata.
    fn metadata(&self) -> RuntimeMetadata;

    /// Declared capabilities.
    fn capabilities(&self) -> RuntimeCapabilities;

    /// Probe host availability (may spawn briefly).
    async fn probe(&self) -> RuntimeStatus;

    /// Execute a program, optionally capturing bindings listed in the request.
    async fn execute(
        &self,
        request: ExecutionRequest,
        policy: &SandboxPolicy,
    ) -> parallax_core::Result<ExecutionResult>;

    /// Restore PIR bindings into a fresh worker context.
    async fn restore(
        &self,
        pir: &PirDocument,
        policy: &SandboxPolicy,
    ) -> parallax_core::Result<RestoreResult>;
}

/// Heap-allocated adapter.
pub type BoxRuntimeAdapter = Arc<dyn RuntimeAdapter>;

/// Factory fn type.
pub type AdapterFactory = Arc<dyn Fn() -> BoxRuntimeAdapter + Send + Sync>;
