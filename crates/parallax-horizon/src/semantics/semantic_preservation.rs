//! Semantic preservation strategies — behavior over syntax.

#![deny(unsafe_code)]
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

/// How to preserve a semantic node when the target has no direct equivalent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreservationStrategy {
    Native,
    Lowered,
    Emulated,
    Wrapped,
    Bridged,
    Capsuled,
    BehaviorSynthesized,
    ManualRequired,
}

impl PreservationStrategy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Native => "NATIVE",
            Self::Lowered => "LOWERED",
            Self::Emulated => "EMULATED",
            Self::Wrapped => "WRAPPED",
            Self::Bridged => "BRIDGED",
            Self::Capsuled => "CAPSULED",
            Self::BehaviorSynthesized => "BEHAVIOR_SYNTHESIZED",
            Self::ManualRequired => "MANUAL_REQUIRED",
        }
    }

    /// Lower is cheaper / preferred when safe.
    pub fn cost_rank(self) -> u8 {
        match self {
            Self::Native => 0,
            Self::Lowered => 1,
            Self::Wrapped => 2,
            Self::Emulated => 3,
            Self::Capsuled => 4,
            Self::Bridged => 5,
            Self::BehaviorSynthesized => 6,
            Self::ManualRequired => 7,
        }
    }
}

/// A hard semantic barrier detected in source.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SemanticBarrier {
    pub id: u32,
    pub kind: String,
    pub location: String,
    pub detail: String,
    pub evidence: Vec<String>,
    pub preferred_strategy: PreservationStrategy,
    pub confidence: f64,
    pub notes: String,
}

/// Decision for one semantic node.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreservationDecision {
    pub node_id: String,
    pub construct: String,
    pub strategy: PreservationStrategy,
    pub rationale: String,
    pub confidence: f64,
    pub capsule_capability: Option<String>,
    pub island_candidate: bool,
}

/// Engine that selects the cheapest safe preservation strategy.
#[derive(Clone, Debug, Default)]
pub struct SemanticPreservationEngine {
    pub policy: PreservationPolicy,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PreservationPolicy {
    #[default]
    MaximumCompatibility,
    MaximumNative,
    MaximumPerformance,
    MinimumDependencies,
    FastestMigration,
}

impl SemanticPreservationEngine {
    pub fn new(policy: PreservationPolicy) -> Self {
        Self { policy }
    }

    /// Choose strategy for a known construct kind.
    pub fn decide(&self, construct: &str, dynamic_score: f64) -> PreservationDecision {
        let (strategy, rationale, capsule, island) = match construct {
            "getattr" | "setattr" | "dynamic_attr" => (
                PreservationStrategy::Capsuled,
                "Dynamic attribute access → specialized DynamicObject capsule".into(),
                Some("dynamic_object".into()),
                false,
            ),
            "eval" | "exec" => (
                if matches!(self.policy, PreservationPolicy::MaximumNative) {
                    PreservationStrategy::ManualRequired
                } else {
                    PreservationStrategy::Bridged
                },
                "eval/exec cannot be natively typed safely; island or manual".into(),
                None,
                true,
            ),
            "method_missing" | "proxy" | "prototype_mutation" => (
                PreservationStrategy::Capsuled,
                "Open dispatch → minified capability capsule from observed call sites".into(),
                Some("open_dispatch".into()),
                false,
            ),
            "reflection" | "Class.forName" => (
                PreservationStrategy::Lowered,
                "Closed reflection sets crystallize into enums/static dispatch".into(),
                None,
                false,
            ),
            "decorator" | "annotation" => (
                PreservationStrategy::Lowered,
                "Translate semantic consequence of decorator, not syntax".into(),
                None,
                false,
            ),
            "monkey_patch" => (
                PreservationStrategy::Emulated,
                "Runtime mutation of types → emulated patch table or island".into(),
                Some("patch_table".into()),
                dynamic_score > 0.8,
            ),
            "c_extension" | "native_addon" | "ffi" => (
                PreservationStrategy::Bridged,
                "Preserve ABI boundary via FFI; do not rewrite native code".into(),
                None,
                false,
            ),
            "asyncio" | "promise" | "goroutine" => (
                PreservationStrategy::Lowered,
                "Concurrency intent → target-native async/runtime".into(),
                None,
                false,
            ),
            _ if dynamic_score >= 0.85 => (
                PreservationStrategy::BehaviorSynthesized,
                "High dynamic usage → observe behavior and synthesize".into(),
                None,
                false,
            ),
            _ => (
                PreservationStrategy::Native,
                "Direct or near-direct target mapping available".into(),
                None,
                false,
            ),
        };

        let strategy = self.apply_policy(strategy);
        PreservationDecision {
            node_id: format!("node:{construct}"),
            construct: construct.into(),
            strategy,
            rationale,
            confidence: (1.0 - dynamic_score * 0.3).clamp(0.4, 0.99),
            capsule_capability: capsule,
            island_candidate: island,
        }
    }

    fn apply_policy(&self, s: PreservationStrategy) -> PreservationStrategy {
        match (&self.policy, s) {
            (PreservationPolicy::MaximumNative, PreservationStrategy::Emulated) => {
                PreservationStrategy::Capsuled
            }
            (PreservationPolicy::MaximumNative, PreservationStrategy::Bridged) => {
                PreservationStrategy::ManualRequired
            }
            (PreservationPolicy::FastestMigration, PreservationStrategy::BehaviorSynthesized) => {
                PreservationStrategy::Bridged
            }
            _ => s,
        }
    }

    /// Prefer cheapest among candidates that meet min confidence.
    pub fn select_cheapest(
        &self,
        options: &[PreservationStrategy],
        min_confidence: f64,
        confidence: f64,
    ) -> PreservationStrategy {
        if confidence < min_confidence {
            return PreservationStrategy::ManualRequired;
        }
        options
            .iter()
            .copied()
            .min_by_key(|s| s.cost_rank())
            .unwrap_or(PreservationStrategy::ManualRequired)
    }
}
