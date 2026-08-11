//! Universal Execution State document.

use crate::deterministic::DeterministicContext;
use crate::frame::UniversalFrame;
use crate::version::{check_ues_format, UES_FORMAT_VERSION};
use indexmap::IndexMap;
use parallax_core::{ExceptionInfo, ExecutionId, RuntimeKind};
use parallax_pcir::PcirProgram;
use serde::{Deserialize, Serialize};

/// Control-position summary at capture time.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct ControlState {
    /// Explicit safepoint label when paused at `parallax.checkpoint`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safepoint_label: Option<String>,
    /// Kind of safepoint (`explicit_checkpoint`, `await`, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safepoint_kind: Option<String>,
    /// Opaque instruction / IP string when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instruction_position: Option<String>,
    /// True when execution is suspended (not completed).
    #[serde(default)]
    pub suspended: bool,
    /// Source offset after which resume should continue (byte index into source).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resume_byte_offset: Option<usize>,
}

/// Heap / object-graph payload (PIR document JSON).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct HeapState {
    /// PIR-tagged bindings object (same shape as worker capture).
    #[serde(default)]
    pub bindings: serde_json::Value,
    /// Optional full PIR document JSON.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pir_document: Option<serde_json::Value>,
}

/// Module table entry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModuleState {
    /// Module name / path.
    pub name: String,
    /// Origin runtime (mixed-origin stacks).
    pub runtime: RuntimeKind,
    /// Optional source fingerprint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
}

/// Async / await bookkeeping (mostly Unsupported today).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct AsyncState {
    /// Status: `none`, `awaiting`, `unsupported`.
    pub status: String,
    /// Detail when unsupported or partial.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// Opaque tasks payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tasks: Option<serde_json::Value>,
}

/// Capability tokens / granted rights at capture.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct CapabilityState {
    /// Granted capability names.
    #[serde(default)]
    pub granted: Vec<String>,
    /// Notes about denied / unsupported capabilities.
    #[serde(default)]
    pub notes: Vec<String>,
}

/// External resource that cannot migrate (files, sockets, …).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExternalResource {
    /// Resource kind.
    pub kind: String,
    /// Descriptor / path / id.
    pub descriptor: String,
    /// Whether migration must fail if this is required.
    #[serde(default)]
    pub required: bool,
}

/// Migration bookkeeping attached to a UES.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct MigrationMetadata {
    /// Capture mode (`explicit_checkpoint`, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capture_mode: Option<String>,
    /// Continuum status: `experimental`, `unsupported`, `verified`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuum_status: Option<String>,
    /// Human notes.
    #[serde(default)]
    pub notes: Vec<String>,
    /// Extra fields reserved for later contract versions.
    #[serde(default)]
    pub extra: IndexMap<String, serde_json::Value>,
}

/// Universal Execution State — suspended computation (not value-only PIR).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UniversalExecutionState {
    /// Format version (independent of PIR / PCIR).
    pub format_version: u32,
    /// Execution identity.
    pub execution_id: ExecutionId,
    /// Source runtime that produced this UES.
    pub source_runtime: RuntimeKind,
    /// Source program path / label.
    pub source_program: Option<String>,
    /// Control / safepoint state.
    pub control_state: ControlState,
    /// Call stack (mixed-origin frames allowed in the model).
    pub call_stack: Vec<UniversalFrame>,
    /// Heap / bindings.
    pub heap: HeapState,
    /// Global bindings (PIR-tagged JSON object).
    #[serde(default)]
    pub globals: IndexMap<String, serde_json::Value>,
    /// Modules.
    #[serde(default)]
    pub modules: Vec<ModuleState>,
    /// Pending exception.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exception_state: Option<ExceptionInfo>,
    /// Async state.
    #[serde(default)]
    pub async_state: AsyncState,
    /// Capability state.
    #[serde(default)]
    pub capability_state: CapabilityState,
    /// Non-migratable external resources.
    #[serde(default)]
    pub external_resources: Vec<ExternalResource>,
    /// Deterministic replay context / hooks.
    #[serde(default)]
    pub deterministic_context: DeterministicContext,
    /// Migration metadata.
    #[serde(default)]
    pub migration_metadata: MigrationMetadata,
    /// Optional PCIR for the supported region around the safepoint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pcir: Option<PcirProgram>,
    /// Opaque resume payload for same-runtime restore (e.g. post-checkpoint source).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resume_source: Option<String>,
    /// Free-form extension bag (forward-compatible).
    #[serde(default)]
    pub extensions: IndexMap<String, serde_json::Value>,
}

impl UniversalExecutionState {
    /// Empty suspended shell.
    pub fn empty(runtime: RuntimeKind) -> Self {
        Self {
            format_version: UES_FORMAT_VERSION,
            execution_id: ExecutionId::new(),
            source_runtime: runtime,
            source_program: None,
            control_state: ControlState {
                suspended: true,
                ..ControlState::default()
            },
            call_stack: Vec::new(),
            heap: HeapState::default(),
            globals: IndexMap::new(),
            modules: Vec::new(),
            exception_state: None,
            async_state: AsyncState {
                status: "none".into(),
                detail: None,
                tasks: None,
            },
            capability_state: CapabilityState::default(),
            external_resources: Vec::new(),
            deterministic_context: DeterministicContext::unsupported_engine(),
            migration_metadata: MigrationMetadata {
                continuum_status: Some("experimental".into()),
                ..MigrationMetadata::default()
            },
            pcir: None,
            resume_source: None,
            extensions: IndexMap::new(),
        }
    }

    /// Build a UES for an explicit checkpoint safepoint (supported Continuum subset).
    pub fn checkpoint_shell(
        runtime: RuntimeKind,
        program: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        let program = program.into();
        let label = label.into();
        let mut ues = Self::empty(runtime.clone());
        ues.source_program = Some(program.clone());
        ues.control_state = ControlState {
            safepoint_label: Some(label.clone()),
            safepoint_kind: Some("explicit_checkpoint".into()),
            instruction_position: Some(format!("safepoint:{label}")),
            suspended: true,
            resume_byte_offset: None,
        };
        ues.call_stack.push(UniversalFrame::checkpoint(
            runtime.clone(),
            "__parallax_main",
            program.clone(),
            label.clone(),
            IndexMap::new(),
        ));
        ues.modules.push(ModuleState {
            name: program,
            runtime,
            fingerprint: None,
        });
        ues.pcir = Some(PcirProgram::checkpoint_stub(&label));
        ues.migration_metadata = MigrationMetadata {
            capture_mode: Some("explicit_checkpoint".into()),
            continuum_status: Some("experimental".into()),
            notes: vec![
                "Captured at explicit parallax.checkpoint safepoint".into(),
                "Arbitrary live stack migration is NOT claimed".into(),
            ],
            extra: IndexMap::new(),
        };
        ues
    }

    /// Validate format version and nested PCIR when present.
    pub fn validate(&self) -> parallax_core::Result<()> {
        check_ues_format(self.format_version)?;
        if let Some(pcir) = &self.pcir {
            pcir.validate()?;
        }
        Ok(())
    }
}
