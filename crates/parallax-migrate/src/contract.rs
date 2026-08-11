//! Formal migration contracts for Continuum continuation migration.
//!
//! A contract is analyzed **before** any live resume attempt. If required
//! semantics cannot be satisfied, migration is rejected with a clear report.

use indexmap::IndexMap;
use parallax_core::{
    CapabilityLevel, ErrorCode, ParallaxError, Remediation, RuntimeCapabilities, RuntimeKind,
};
use parallax_ues::{UniversalExecutionState, UES_FORMAT_VERSION};
use serde::{Deserialize, Serialize};

/// Semantics that must survive a continuation migration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequiredSemantic {
    /// Named bindings / heap values.
    Values,
    /// Global bindings.
    Globals,
    /// Local bindings at the safepoint.
    Locals,
    /// Control / safepoint position (not restart-from-top).
    ControlPosition,
    /// At least one stack frame (checkpoint frame).
    StackFrames,
    /// Exception state.
    Exceptions,
    /// Closure environments.
    Closures,
    /// Async / await state.
    AsyncState,
    /// Same-runtime resume after checkpoint.
    SameRuntimeResume,
    /// Cross-runtime resume.
    CrossRuntimeResume,
    /// Deterministic replay.
    DeterministicReplay,
}

impl RequiredSemantic {
    /// Capability name consulted on [`RuntimeCapabilities`].
    pub fn capability_name(&self) -> &'static str {
        match self {
            Self::Values => "values",
            Self::Globals => "globals",
            Self::Locals => "locals",
            Self::ControlPosition => "control_position",
            Self::StackFrames => "stack_frames",
            Self::Exceptions => "exception_state",
            Self::Closures => "closures",
            Self::AsyncState => "async_migration",
            Self::SameRuntimeResume => "continuation_restore",
            Self::CrossRuntimeResume => "cross_runtime_resume",
            Self::DeterministicReplay => "continuation_restore", // replay uses separate engine check
        }
    }
}

/// Formal contract describing required surviving semantics for a migration.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MigrationContract {
    /// Contract schema version (reserved for future fields).
    pub version: u32,
    /// Source runtime.
    pub source: RuntimeKind,
    /// Target runtime.
    pub target: RuntimeKind,
    /// Migration mode (`value` PIR vs `continuation` UES).
    pub mode: String,
    /// Required semantics that must remain usable on the target.
    pub required: Vec<RequiredSemantic>,
    /// Whether experimental capabilities may satisfy requirements.
    #[serde(default = "default_true")]
    pub allow_experimental: bool,
    /// Whether partial capabilities may satisfy requirements.
    #[serde(default)]
    pub allow_partial: bool,
    /// Optional UES format version expected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_ues_format: Option<u32>,
    /// Reserved extension bag for later contract fields.
    #[serde(default)]
    pub extensions: IndexMap<String, serde_json::Value>,
}

fn default_true() -> bool {
    true
}

impl MigrationContract {
    /// Default value-migration contract (PIR bindings only).
    pub fn value_migration(source: RuntimeKind, target: RuntimeKind) -> Self {
        Self {
            version: 1,
            source,
            target,
            mode: "value".into(),
            required: vec![RequiredSemantic::Values, RequiredSemantic::Globals],
            allow_experimental: true,
            allow_partial: true,
            expected_ues_format: None,
            extensions: IndexMap::new(),
        }
    }

    /// Continuation migration contract for explicit-checkpoint Continuum paths.
    pub fn continuation_checkpoint(source: RuntimeKind, target: RuntimeKind) -> Self {
        let mut required = vec![
            RequiredSemantic::Values,
            RequiredSemantic::Locals,
            RequiredSemantic::ControlPosition,
            RequiredSemantic::StackFrames,
            RequiredSemantic::SameRuntimeResume,
        ];
        if source != target {
            required.push(RequiredSemantic::CrossRuntimeResume);
        }
        Self {
            version: 1,
            source,
            target,
            mode: "continuation".into(),
            required,
            allow_experimental: true,
            // Locals are PARTIAL on Python/JS by design (checkpoint subset).
            allow_partial: true,
            expected_ues_format: Some(UES_FORMAT_VERSION),
            extensions: IndexMap::new(),
        }
    }
}

/// One finding from contract analysis.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContractFinding {
    /// Required semantic.
    pub semantic: RequiredSemantic,
    /// Capability consulted.
    pub capability: String,
    /// Level on the target.
    pub level: CapabilityLevel,
    /// Whether this finding blocks the contract.
    pub blocking: bool,
    /// Message.
    pub message: String,
}

/// Result of analyzing a [`MigrationContract`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContractAnalysis {
    /// Contract under analysis.
    pub contract: MigrationContract,
    /// Findings.
    pub findings: Vec<ContractFinding>,
    /// Whether the contract is satisfied.
    pub satisfied: bool,
    /// Overall status: `ok`, `rejected`, `unsupported`.
    pub status: String,
    /// Human notes.
    #[serde(default)]
    pub notes: Vec<String>,
}

impl ContractAnalysis {
    /// Human-readable report.
    pub fn format_human(&self) -> String {
        let mut out = format!(
            "MigrationContract v{}  {} → {}  mode={}  status={}\n",
            self.contract.version,
            self.contract.source,
            self.contract.target,
            self.contract.mode,
            self.status
        );
        for f in &self.findings {
            let mark = if f.blocking { "BLOCK" } else { "ok" };
            out.push_str(&format!(
                "  [{mark}] {:?} via {} = {} — {}\n",
                f.semantic, f.capability, f.level.glyph(), f.message
            ));
        }
        for n in &self.notes {
            out.push_str(&format!("  note: {n}\n"));
        }
        out
    }
}

fn level_acceptable(level: CapabilityLevel, contract: &MigrationContract) -> bool {
    match level {
        CapabilityLevel::Yes => true,
        CapabilityLevel::Partial => contract.allow_partial,
        CapabilityLevel::Experimental => contract.allow_experimental,
        CapabilityLevel::No => false,
    }
}

fn caps_for(runtime: &RuntimeKind) -> RuntimeCapabilities {
    match runtime {
        RuntimeKind::Python => RuntimeCapabilities::python(),
        RuntimeKind::JavaScript => RuntimeCapabilities::javascript(),
        RuntimeKind::Wasm => RuntimeCapabilities::wasm(),
        RuntimeKind::Other(_) => RuntimeCapabilities::none(),
    }
}

/// Analyze a migration contract against declared target capabilities.
///
/// Must run before any live continuation resume attempt.
pub fn analyze_contract(contract: &MigrationContract) -> ContractAnalysis {
    let target_caps = caps_for(&contract.target);
    let mut findings = Vec::new();
    let mut notes = Vec::new();

    if contract.mode == "continuation" {
        notes.push(
            "Continuation mode requires a checkpoint-produced UES; arbitrary live stacks Unsupported"
                .into(),
        );
        if contract.source != contract.target {
            notes.push(
                "Cross-runtime continuation resume is currently Unsupported (EXP gate only)"
                    .into(),
            );
        }
    }

    for semantic in &contract.required {
        if *semantic == RequiredSemantic::DeterministicReplay {
            findings.push(ContractFinding {
                semantic: semantic.clone(),
                capability: "deterministic_replay".into(),
                level: CapabilityLevel::No,
                blocking: true,
                message: "Deterministic replay engine is not implemented".into(),
            });
            continue;
        }
        let cap = semantic.capability_name();
        let level = target_caps.level_of(cap).unwrap_or(CapabilityLevel::No);
        let acceptable = level_acceptable(level, contract);
        // Cross-runtime resume is never silently accepted when level is No.
        let blocking = !acceptable;
        findings.push(ContractFinding {
            semantic: semantic.clone(),
            capability: cap.into(),
            level,
            blocking,
            message: if blocking {
                format!("required semantic {:?} not satisfied on {}", semantic, contract.target)
            } else {
                format!("satisfied at {}", level.glyph())
            },
        });
    }

    let blocked = findings.iter().any(|f| f.blocking);
    let status = if blocked {
        if findings.iter().any(|f| {
            f.blocking && matches!(f.level, CapabilityLevel::No) && f.capability.contains("cross")
        }) {
            "unsupported"
        } else {
            "rejected"
        }
    } else {
        "ok"
    };

    ContractAnalysis {
        contract: contract.clone(),
        findings,
        satisfied: !blocked,
        status: status.into(),
        notes,
    }
}

/// Analyze contract and reject with a structured error when unsatisfied.
pub fn require_contract(contract: &MigrationContract) -> parallax_core::Result<ContractAnalysis> {
    let analysis = analyze_contract(contract);
    if analysis.satisfied {
        return Ok(analysis);
    }
    let detail = analysis.format_human();
    Err(ParallaxError::new(
        ErrorCode::MigrationRejected,
        format!(
            "migration contract not satisfied ({} → {}, mode={})",
            contract.source, contract.target, contract.mode
        ),
    )
    .with_source("parallax-migrate")
    .with_operation("require_contract")
    .context("status", analysis.status.clone())
    .context("report", detail)
    .remediate(Remediation::with_detail(
        "Inspect the contract report; use value migration or same-runtime checkpoint resume",
        "Cross-runtime continuation resume is Unsupported in this Continuum milestone",
    )))
}

/// Validate a UES against a continuation contract (format + source runtime).
pub fn analyze_ues_contract(
    contract: &MigrationContract,
    ues: &UniversalExecutionState,
) -> ContractAnalysis {
    let mut analysis = analyze_contract(contract);
    if let Some(expected) = contract.expected_ues_format {
        if ues.format_version != expected {
            analysis.satisfied = false;
            analysis.status = "rejected".into();
            analysis.findings.push(ContractFinding {
                semantic: RequiredSemantic::ControlPosition,
                capability: "ues_format".into(),
                level: CapabilityLevel::No,
                blocking: true,
                message: format!(
                    "UES format {} != expected {}",
                    ues.format_version, expected
                ),
            });
        }
    }
    if ues.source_runtime != contract.source {
        analysis.satisfied = false;
        analysis.status = "rejected".into();
        analysis.notes.push(format!(
            "UES source_runtime {} != contract source {}",
            ues.source_runtime, contract.source
        ));
    }
    if contract.mode == "continuation"
        && ues.control_state.safepoint_kind.as_deref() != Some("explicit_checkpoint")
    {
        analysis.satisfied = false;
        analysis.status = "unsupported".into();
        analysis.findings.push(ContractFinding {
            semantic: RequiredSemantic::ControlPosition,
            capability: "control_position".into(),
            level: CapabilityLevel::No,
            blocking: true,
            message: "UES was not produced at an explicit checkpoint safepoint".into(),
        });
    }
    if !analysis.satisfied && analysis.status == "ok" {
        analysis.status = "rejected".into();
    }
    analysis
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_runtime_python_checkpoint_ok() {
        let c = MigrationContract::continuation_checkpoint(
            RuntimeKind::Python,
            RuntimeKind::Python,
        );
        let a = analyze_contract(&c);
        assert!(a.satisfied, "{}", a.format_human());
    }

    #[test]
    fn cross_runtime_continuation_rejected() {
        let c = MigrationContract::continuation_checkpoint(
            RuntimeKind::Python,
            RuntimeKind::JavaScript,
        );
        let a = analyze_contract(&c);
        assert!(!a.satisfied);
        assert!(matches!(a.status.as_str(), "rejected" | "unsupported"));
        let err = require_contract(&c).unwrap_err();
        assert_eq!(err.code, ErrorCode::MigrationRejected);
    }

    #[test]
    fn wasm_continuation_unsupported() {
        let c =
            MigrationContract::continuation_checkpoint(RuntimeKind::Wasm, RuntimeKind::Wasm);
        let a = analyze_contract(&c);
        assert!(!a.satisfied);
    }
}
