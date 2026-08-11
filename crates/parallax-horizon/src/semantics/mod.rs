//! Preservation strategies, capsules, polyglot islands, debt, metaprogram lowering.

mod capsule;
mod debt;
mod metaprogram;
mod polyglot;
mod semantic_preservation;
mod strategy;

pub use capsule::{
    CapsuleCapability, CapsuleFile, CapsuleGenerator, CapsuleSpec, GeneratedCapsule,
    SemanticMinifier,
};
pub use debt::{DebtBreakdown, DebtTracker, EvolutionPoint};
pub use metaprogram::{ExpandedMethod, LoweredDecorator, MetaprogramExpander};
pub use polyglot::{
    BoundarySynthesizer, DissolveReport, IslandBoundary, IslandBoundaryKind, IslandDissolver,
    IslandEntrypoint, PolyglotIsland,
};
pub use semantic_preservation::{
    PreservationDecision, PreservationPolicy, PreservationStrategy, SemanticBarrier,
    SemanticPreservationEngine,
};
pub use strategy::{SemanticHazard, SemanticHazardDatabase, StrategyOption, StrategySearcher};
