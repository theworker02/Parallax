//! Continuation / Continuum capability matrix (honest EXP / PARTIAL / NO).

use parallax_core::{CapabilityLevel, RuntimeCapabilities, RuntimeKind};
use serde::{Deserialize, Serialize};

/// One row in the continuation matrix.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContinuationCapabilityRow {
    /// Canonical snake_case name.
    pub name: String,
    /// Display label.
    pub label: String,
    /// Support level.
    pub level: CapabilityLevel,
    /// Short note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Continuation-focused capability matrix for a runtime.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContinuationCapabilityMatrix {
    /// Runtime.
    pub runtime: RuntimeKind,
    /// Rows.
    pub rows: Vec<ContinuationCapabilityRow>,
}

impl ContinuationCapabilityMatrix {
    /// Human table.
    pub fn format_human(&self) -> String {
        let mut out = format!("[{}] continuation capabilities\n", self.runtime);
        for row in &self.rows {
            let note = row
                .note
                .as_deref()
                .map(|n| format!("  — {n}"))
                .unwrap_or_default();
            out.push_str(&format!(
                "  {:<28} {}{}\n",
                row.label,
                row.level.glyph(),
                note
            ));
        }
        out
    }
}

fn row(
    name: &str,
    label: &str,
    level: CapabilityLevel,
    note: Option<&str>,
) -> ContinuationCapabilityRow {
    ContinuationCapabilityRow {
        name: name.into(),
        label: label.into(),
        level,
        note: note.map(|s| s.into()),
    }
}

/// Build the Continuum continuation matrix for a runtime.
pub fn continuation_matrix(runtime: RuntimeKind) -> ContinuationCapabilityMatrix {
    let caps = match &runtime {
        RuntimeKind::Python => RuntimeCapabilities::python(),
        RuntimeKind::JavaScript => RuntimeCapabilities::javascript(),
        RuntimeKind::Wasm => RuntimeCapabilities::wasm(),
        RuntimeKind::Other(_) => RuntimeCapabilities::none(),
    };
    ContinuationCapabilityMatrix {
        runtime: runtime.clone(),
        rows: vec![
            row(
                "stack_frames",
                "Stack frames",
                caps.stack_frames,
                Some("Arbitrary frames not captured; checkpoint frame only"),
            ),
            row(
                "control_position",
                "Control position",
                caps.control_position,
                Some("Safepoint label / resume offset when checkpoint hits"),
            ),
            row(
                "exceptions",
                "Exceptions in flight",
                caps.exception_state,
                None,
            ),
            row("closures", "Closures", caps.closures, None),
            row("async_state", "Async state", caps.async_migration, None),
            row(
                "continuation_capture",
                "Continuation capture",
                caps.continuation_capture,
                Some("Via explicit parallax.checkpoint() only"),
            ),
            row(
                "continuation_restore",
                "Continuation restore",
                caps.continuation_restore,
                Some("Same-runtime checkpoint resume only when Experimental"),
            ),
            row(
                "explicit_checkpoint_capture",
                "Explicit checkpoint capture",
                match runtime {
                    RuntimeKind::Python | RuntimeKind::JavaScript => {
                        CapabilityLevel::Experimental
                    }
                    _ => CapabilityLevel::No,
                },
                Some("Real UES at parallax.checkpoint"),
            ),
            row(
                "same_runtime_resume",
                "Same-runtime resume",
                match runtime {
                    RuntimeKind::Python | RuntimeKind::JavaScript => {
                        CapabilityLevel::Experimental
                    }
                    _ => CapabilityLevel::No,
                },
                Some("Resume post-checkpoint source with restored bindings"),
            ),
            row(
                "cross_runtime_resume",
                "Cross-runtime resume",
                caps.cross_runtime_resume,
                Some("Contract-gated; currently Unsupported"),
            ),
        ],
    }
}
