//! Migration report types.

use crate::analyze::LossFinding;
use parallax_core::{ConversionPolicy, MigrationId, RuntimeKind, SemanticLoss};
use serde::{Deserialize, Serialize};

/// Measured timings for a migration (microseconds; never fabricated).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MigrationTimings {
    /// Analysis duration.
    pub analyze_us: u64,
    /// Conversion duration.
    pub convert_us: u64,
    /// Capture duration (when live adapters used).
    pub capture_us: Option<u64>,
    /// Restore duration (when live adapters used).
    pub restore_us: Option<u64>,
    /// Total measured duration.
    pub total_us: u64,
}

/// Full migration report.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MigrationReport {
    /// Migration id.
    pub id: MigrationId,
    /// Source runtime.
    pub source_runtime: RuntimeKind,
    /// Target runtime.
    pub target_runtime: RuntimeKind,
    /// Policy used.
    pub policy: ConversionPolicy,
    /// Findings.
    pub findings: Vec<LossFinding>,
    /// Worst loss observed.
    pub worst_loss: SemanticLoss,
    /// Timings.
    pub timings: MigrationTimings,
    /// Success flag.
    pub success: bool,
    /// Extra notes.
    pub notes: Vec<String>,
}

impl MigrationReport {
    /// Human-readable summary.
    pub fn format_human(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "Migration {} → {} ({})\n",
            self.source_runtime,
            self.target_runtime,
            if self.success { "OK" } else { "FAILED" }
        ));
        out.push_str(&format!("id: {}\n", self.id));
        out.push_str(&format!("worst_loss: {}\n", self.worst_loss));
        out.push_str("timings:\n");
        if let Some(c) = self.timings.capture_us {
            out.push_str(&format!("  capture: {c} µs\n"));
        }
        out.push_str(&format!("  analyze: {} µs\n", self.timings.analyze_us));
        out.push_str(&format!("  convert: {} µs\n", self.timings.convert_us));
        if let Some(r) = self.timings.restore_us {
            out.push_str(&format!("  restore: {r} µs\n"));
        }
        out.push_str(&format!("  total:   {} µs\n", self.timings.total_us));
        if !self.findings.is_empty() {
            out.push_str("\nfindings:\n");
            for f in &self.findings {
                out.push_str(&format!("  [{}] {} — {}\n", f.loss, f.path, f.message));
                if let Some(s) = &f.suggestion {
                    out.push_str(&format!("      suggestion: {s}\n"));
                }
            }
        }
        for n in &self.notes {
            out.push_str(&format!("note: {n}\n"));
        }
        out
    }
}
