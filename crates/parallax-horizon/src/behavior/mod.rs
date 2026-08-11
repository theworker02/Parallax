//! Project observation, behavioral contracts, exploration, synthesis, CEGIS repair.

mod contracts;
mod counterexample;
mod exploration;
mod observer;
mod synthesis;

pub use exploration::{
    BehaviorExplorer, ExplorationPlan, SemanticFuzzCase, SemanticFuzzer, SemanticTriangulator,
    TriangulationReport,
};
pub use contracts::{BehaviorObservation, BehavioralContract};
pub use counterexample::{CegisLoop, Counterexample, RepairStep};
pub use observer::{DynamicSignal, ObservatoryReport, ProjectObserver};
pub use synthesis::{BehaviorSynthesizer, SynthesisRequest, SynthesisResult};
