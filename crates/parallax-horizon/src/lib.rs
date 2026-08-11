//! Event Horizon — semantic reconstruction for "impossible" migrations.
//!
//! Philosophy: no direct equivalent ≠ migration impossible.
//! Preserve behavior via native / capsule / island / synthesis — never silent drift.

#![deny(unsafe_code)]
#![allow(missing_docs)]

mod plan;

pub mod behavior;
pub mod ir;
pub mod pvabi;
pub mod semantics;
pub mod vcs;

pub use plan::{
    analyze_impossible, blame_line, cherry_pick, detach_status, dissolve_project, explain_barrier,
    measure_debt, optimize_migration, reconstruct_status, HorizonPlan, ImpossibleReport,
};

// Re-export commonly used types at crate root for ergonomic CLI / library use.
pub use behavior::{
    BehaviorExplorer, BehaviorObservation, BehavioralContract, CegisLoop, Counterexample,
    DynamicSignal, ExplorationPlan, ObservatoryReport, ProjectObserver, RepairStep,
    SemanticFuzzCase, SemanticFuzzer, SemanticTriangulator, SynthesisRequest, SynthesisResult,
    TriangulationReport,
};
pub use ir::{
    ConcurrencyGraph, ConcurrencyIntent, CrystallizedKind, CrystallizedType, FieldType, HttpRoute,
    MessageShape, ProtocolIr, QueryIr, QueryStmt, Shape, ShapeInferencer, TypeCrystallizer,
};
pub use semantics::{
    BoundarySynthesizer, CapsuleCapability, CapsuleFile, CapsuleGenerator, CapsuleSpec,
    DebtBreakdown, DebtTracker, DissolveReport, EvolutionPoint, ExpandedMethod, GeneratedCapsule,
    IslandBoundary, IslandBoundaryKind, IslandDissolver, IslandEntrypoint, LoweredDecorator,
    MetaprogramExpander, PolyglotIsland, PreservationDecision, PreservationPolicy,
    PreservationStrategy, SemanticBarrier, SemanticHazard, SemanticHazardDatabase,
    SemanticMinifier, SemanticPreservationEngine, StrategyOption, StrategySearcher,
};
pub use vcs::{PatchOp, SemanticBlame, SemanticGit, SemanticPatch, PLXP_FORMAT_VERSION};

use parallax_core::PARALLAX_VERSION;

/// Event Horizon format version.
pub const HORIZON_FORMAT_VERSION: u32 = 1;

/// Version snapshot for CLI / lockfiles.
pub fn versions() -> serde_json::Value {
    serde_json::json!({
        "horizon_format": HORIZON_FORMAT_VERSION,
        "parallax": PARALLAX_VERSION,
        "pvabi": pvabi::PVABI_SCHEMA_VERSION,
        "plxp": PLXP_FORMAT_VERSION,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn impossible_detects_getattr() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("app.py"),
            "def f(o, n):\n    return getattr(o, n)\n",
        )
        .unwrap();
        fs::write(dir.path().join("requirements.txt"), "fastapi\n").unwrap();
        let report = analyze_impossible(dir.path(), Some("rust"), None).unwrap();
        assert!(!report.barriers.is_empty());
        assert!(report.estimated_native_pct > 50.0);
        assert!(report
            .proposed
            .iter()
            .any(|p| p.contains("getattr") || p.contains("Dynamic")));
    }
}
