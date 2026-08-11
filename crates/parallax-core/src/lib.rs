//! Parallax core types: errors, capabilities, runtime metadata, and execution model.
//!
//! This crate is intentionally free of adapter-specific logic.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod capabilities;
pub mod error;
pub mod execution;
pub mod ids;
pub mod runtime;
pub mod semantic;
pub mod version;

pub use capabilities::{CapabilityLevel, RuntimeCapabilities};
pub use error::{ErrorCode, ParallaxError, Remediation, Result};
pub use execution::{
    ExceptionInfo, ExecutionLimits, ExecutionRequest, ExecutionResult, ExecutionState,
    ProgramSource, RestoreResult,
};
pub use ids::{ExecutionId, MigrationId, ObjectId, RequestId, RuntimeId, SnapshotId};
pub use runtime::{RuntimeKind, RuntimeMetadata, RuntimeStatus};
pub use semantic::{
    integer_to_js_number_loss, ConversionPolicy, SemanticLoss, JS_MAX_SAFE_INTEGER,
    JS_MIN_SAFE_INTEGER,
};
pub use version::{
    check_adapter_interface, check_pcir_schema, check_pir_schema, check_protocol,
    check_snapshot_format, check_ues_format, ComponentVersions, ADAPTER_INTERFACE_VERSION,
    MIRROR_LINK_FORMAT_VERSION, PARALLAX_VERSION, PCIR_SCHEMA_VERSION, PIR_SCHEMA_VERSION,
    PROTOCOL_VERSION, PUIR_SCHEMA_VERSION, SNAPSHOT_FORMAT_VERSION, UES_FORMAT_VERSION,
};
