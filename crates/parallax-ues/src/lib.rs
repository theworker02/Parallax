//! Universal Execution State (UES) — Continuum foundation.
//!
//! **PIR** models portable **values**. **UES** models suspended **execution**:
//! control position, frames, heap roots, capability/external resource state, and
//! optional PCIR for supported regions.
//!
//! This crate provides types, versioning, safepoint reports, deterministic-replay
//! hooks, and continuation capability matrices. Live cross-runtime stack migration
//! is capability-gated and must not be faked.

#![deny(unsafe_code)]
#![warn(missing_docs)]

mod capabilities;
mod deterministic;
mod frame;
mod safepoint;
mod serde_bin;
mod state;
mod version;

pub use capabilities::{
    continuation_matrix, ContinuationCapabilityMatrix, ContinuationCapabilityRow,
};
pub use deterministic::{
    DeterministicContext, ReplayEngineStatus, ReplayJournal, ReplayJournalEntry,
};
pub use frame::{FrameId, SourceLocation, UniversalFrame};
pub use safepoint::{
    SafepointKind, SafepointReport, SemanticLossNote, CAPTURE_YES, MIGRATE_NO, MIGRATE_PARTIAL,
    REPLAY_UNSUPPORTED, SNAPSHOT_YES,
};
pub use serde_bin::{from_binary, to_binary, UES_MAGIC};
pub use state::{
    AsyncState, CapabilityState, ControlState, ExternalResource, HeapState, MigrationMetadata,
    ModuleState, UniversalExecutionState,
};
pub use version::{check_ues_format, UES_FORMAT_VERSION};

use parallax_core::{ErrorCode, ParallaxError};

/// Result alias.
pub type Result<T> = parallax_core::Result<T>;

/// Serialize UES to pretty JSON bytes.
pub fn to_json_bytes(ues: &UniversalExecutionState) -> Result<Vec<u8>> {
    serde_json::to_vec_pretty(ues).map_err(|e| {
        ParallaxError::new(ErrorCode::SerializationFailure, e.to_string())
            .with_source("parallax-ues")
            .with_operation("to_json_bytes")
    })
}

/// Serialize UES to compact JSON bytes.
pub fn to_json_bytes_compact(ues: &UniversalExecutionState) -> Result<Vec<u8>> {
    serde_json::to_vec(ues).map_err(|e| {
        ParallaxError::new(ErrorCode::SerializationFailure, e.to_string())
            .with_source("parallax-ues")
            .with_operation("to_json_bytes_compact")
    })
}

/// Deserialize and validate UES from JSON.
pub fn from_json_bytes(bytes: &[u8]) -> Result<UniversalExecutionState> {
    let ues: UniversalExecutionState = serde_json::from_slice(bytes).map_err(|e| {
        ParallaxError::new(ErrorCode::SerializationFailure, e.to_string())
            .with_source("parallax-ues")
            .with_operation("from_json_bytes")
    })?;
    ues.validate()?;
    Ok(ues)
}

#[cfg(test)]
mod tests {
    use super::*;
    use parallax_core::RuntimeKind;
    use parallax_pcir::PcirProgram;

    #[test]
    fn json_round_trip_and_version() {
        let mut ues =
            UniversalExecutionState::checkpoint_shell(RuntimeKind::Python, "demo.py", "cp1");
        ues.pcir = Some(PcirProgram::checkpoint_stub("cp1"));
        let bytes = to_json_bytes(&ues).unwrap();
        let back = from_json_bytes(&bytes).unwrap();
        assert_eq!(back.format_version, UES_FORMAT_VERSION);
        assert_eq!(back.control_state.safepoint_label.as_deref(), Some("cp1"));
    }

    #[test]
    fn binary_round_trip() {
        let ues =
            UniversalExecutionState::checkpoint_shell(RuntimeKind::JavaScript, "demo.js", "mark");
        let bin = to_binary(&ues).unwrap();
        assert!(bin.starts_with(UES_MAGIC));
        let back = from_binary(&bin).unwrap();
        assert_eq!(back.source_runtime, RuntimeKind::JavaScript);
    }

    #[test]
    fn rejects_future_ues_version() {
        let mut ues = UniversalExecutionState::checkpoint_shell(RuntimeKind::Python, "x.py", "a");
        ues.format_version = UES_FORMAT_VERSION + 9;
        let bytes = to_json_bytes_compact(&ues).unwrap();
        let err = from_json_bytes(&bytes).unwrap_err();
        assert!(err.message.contains("unsupported UES format"));
    }

    #[test]
    fn continuation_matrix_is_honest() {
        let m = continuation_matrix(RuntimeKind::Python);
        assert!(m
            .rows
            .iter()
            .any(|r| r.name == "explicit_checkpoint_capture"));
        let wasm = continuation_matrix(RuntimeKind::Wasm);
        let cross = wasm
            .rows
            .iter()
            .find(|r| r.name == "cross_runtime_resume")
            .unwrap();
        assert_eq!(cross.level, parallax_core::CapabilityLevel::No);
    }
}
