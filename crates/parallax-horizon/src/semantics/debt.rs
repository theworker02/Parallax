//! Compatibility debt scoring and burn-down tracking.

#![deny(unsafe_code)]
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebtBreakdown {
    pub native_pct: f64,
    pub compatibility_pct: f64,
    pub polyglot_island_pct: f64,
    pub manual_pct: f64,
    pub target_purity: f64,
    pub notes: String,
}

impl DebtBreakdown {
    pub fn from_parts(native: f64, compat: f64, island: f64, manual: f64) -> Self {
        let sum = native + compat + island + manual;
        let norm = if sum <= 0.0 { 1.0 } else { sum };
        let native_pct = native / norm * 100.0;
        let compatibility_pct = compat / norm * 100.0;
        let polyglot_island_pct = island / norm * 100.0;
        let manual_pct = manual / norm * 100.0;
        Self {
            target_purity: native_pct,
            native_pct,
            compatibility_pct,
            polyglot_island_pct,
            manual_pct,
            notes: "Debt scores are measured from preservation decisions — not LOC vanity metrics"
                .into(),
        }
    }

    pub fn can_detach(&self) -> bool {
        self.polyglot_island_pct < 0.05
            && self.compatibility_pct < 0.5
            && self.manual_pct < 0.05
            && self.native_pct >= 99.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvolutionPoint {
    pub label: String,
    pub native_pct: f64,
}

#[derive(Clone, Debug, Default)]
pub struct DebtTracker;

impl DebtTracker {
    pub fn burn_down_hint(&self, debt: &DebtBreakdown) -> Vec<String> {
        let mut hints = Vec::new();
        if debt.polyglot_island_pct > 0.0 {
            hints.push("Run `plx dissolve` to shrink polyglot islands".into());
        }
        if debt.compatibility_pct > 5.0 {
            hints.push("Run `plx optimize-migration` to replace capsules with native types".into());
        }
        if debt.manual_pct > 0.0 {
            hints.push("Resolve MANUAL_REQUIRED barriers via `plx explain-barrier`".into());
        }
        if hints.is_empty() {
            hints.push("Debt is low — consider `plx detach` after verification gates".into());
        }
        hints
    }
}
