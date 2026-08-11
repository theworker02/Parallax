//! Parallax safepoint model and machine-readable reports.

use indexmap::IndexMap;
use parallax_core::{CapabilityLevel, RuntimeKind};
use serde::{Deserialize, Serialize};

/// Capture is possible at this safepoint.
pub const CAPTURE_YES: &str = "YES";
/// Snapshot of values/UES is possible.
pub const SNAPSHOT_YES: &str = "YES";
/// Deterministic replay engine is not ready.
pub const REPLAY_UNSUPPORTED: &str = "UNSUPPORTED";
/// Cross-runtime migrate of live stacks is not available.
pub const MIGRATE_NO: &str = "NO";
/// Partial migrate (same-runtime checkpoint resume only).
pub const MIGRATE_PARTIAL: &str = "PARTIAL";

/// Kind of Continuum safepoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafepointKind {
    /// Function entry / exit boundary.
    Function,
    /// Loop back-edge.
    Loop,
    /// Await suspension point.
    Await,
    /// Generator / coroutine yield.
    Yield,
    /// Explicit `parallax.checkpoint()` / `@parallax.safepoint`.
    ExplicitCheckpoint,
}

/// Semantic-loss note attached to a safepoint report.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticLossNote {
    /// Short code.
    pub code: String,
    /// Human message.
    pub message: String,
}

/// Machine-readable report produced at a safepoint.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SafepointReport {
    /// Safepoint kind.
    pub kind: SafepointKind,
    /// Label / name.
    pub label: String,
    /// Runtime that hit the safepoint.
    pub runtime: RuntimeKind,
    /// Whether capture of UES is possible.
    pub can_capture: String,
    /// Whether a durable snapshot can be written.
    pub can_snapshot: String,
    /// Whether deterministic replay is available.
    pub can_replay: String,
    /// Whether migration off this safepoint is available.
    pub can_migrate: String,
    /// Candidate target runtimes for resume (may be empty).
    pub targets: Vec<RuntimeKind>,
    /// Declared capability levels for this path.
    pub capability_levels: IndexMap<String, CapabilityLevel>,
    /// Semantic loss expected if migrated.
    #[serde(default)]
    pub semantic_loss: Vec<SemanticLossNote>,
    /// Honest status banner.
    pub status: String,
    /// Extra diagnostics.
    #[serde(default)]
    pub notes: Vec<String>,
}

impl SafepointReport {
    /// Report for an explicit checkpoint on a runtime that supports capture.
    pub fn explicit_checkpoint(runtime: RuntimeKind, label: impl Into<String>) -> Self {
        let label = label.into();
        let (can_migrate, targets, status) = match runtime {
            RuntimeKind::Python | RuntimeKind::JavaScript => (
                MIGRATE_PARTIAL.to_string(),
                vec![runtime.clone()],
                "EXPERIMENTAL".to_string(),
            ),
            _ => (
                MIGRATE_NO.to_string(),
                Vec::new(),
                "UNSUPPORTED".to_string(),
            ),
        };
        let mut levels = IndexMap::new();
        levels.insert("capture".into(), CapabilityLevel::Experimental);
        levels.insert("snapshot".into(), CapabilityLevel::Experimental);
        levels.insert("replay".into(), CapabilityLevel::No);
        levels.insert(
            "same_runtime_resume".into(),
            if matches!(runtime, RuntimeKind::Python | RuntimeKind::JavaScript) {
                CapabilityLevel::Experimental
            } else {
                CapabilityLevel::No
            },
        );
        levels.insert("cross_runtime_resume".into(), CapabilityLevel::No);
        Self {
            kind: SafepointKind::ExplicitCheckpoint,
            label,
            runtime,
            can_capture: CAPTURE_YES.into(),
            can_snapshot: SNAPSHOT_YES.into(),
            can_replay: REPLAY_UNSUPPORTED.into(),
            can_migrate,
            targets,
            capability_levels: levels,
            semantic_loss: vec![SemanticLossNote {
                code: "no_live_stack".into(),
                message: "Only explicit checkpoint regions are captured; arbitrary frames are not"
                    .into(),
            }],
            status,
            notes: vec![
                "Hit via parallax.checkpoint() / @parallax.safepoint".into(),
                "Cross-runtime continuation resume is Unsupported".into(),
            ],
        }
    }

    /// Human-readable summary.
    pub fn format_human(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "safepoint '{}' ({:?}) on {} — {}\n",
            self.label, self.kind, self.runtime, self.status
        ));
        out.push_str(&format!(
            "  capture={}  snapshot={}  replay={}  migrate={}\n",
            self.can_capture, self.can_snapshot, self.can_replay, self.can_migrate
        ));
        if !self.targets.is_empty() {
            let t = self
                .targets
                .iter()
                .map(|r| r.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("  targets: {t}\n"));
        }
        for n in &self.notes {
            out.push_str(&format!("  note: {n}\n"));
        }
        out
    }
}
