//! Known cross-language semantic hazards + migration strategy search.

#![deny(unsafe_code)]
#![allow(missing_docs)]

use super::semantic_preservation::{PreservationPolicy, PreservationStrategy};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SemanticHazard {
    pub id: String,
    pub title: String,
    pub source_langs: Vec<String>,
    pub target_langs: Vec<String>,
    pub detail: String,
    pub mitigation: String,
}

#[derive(Clone, Debug, Default)]
pub struct SemanticHazardDatabase;

impl SemanticHazardDatabase {
    pub fn builtins(&self) -> Vec<SemanticHazard> {
        vec![
            h(
                "js-number-precision",
                "JavaScript Number precision",
                &["javascript", "typescript"],
                &["rust", "go", "python"],
                "IEEE-754 doubles cannot represent all integers above 2^53-1",
                "Use BigInt / i64 / stringified integers",
            ),
            h(
                "py-int-arbitrary",
                "Python arbitrary-size integers",
                &["python"],
                &["rust", "javascript"],
                "int is unbounded in CPython",
                "Prove bounds or use BigInt / decimal",
            ),
            h(
                "rust-overflow",
                "Rust overflow behavior",
                &["python", "javascript"],
                &["rust"],
                "Debug panics / release wraps for overflow",
                "Use checked_* or wrapping intentionally",
            ),
            h(
                "go-zero-values",
                "Go zero values",
                &["python", "typescript"],
                &["go"],
                "Unset fields are zero, not nil/None",
                "Model optionality with pointers or ok-idiom",
            ),
            h(
                "js-coercion",
                "JavaScript coercion",
                &["javascript", "typescript"],
                &["rust", "go"],
                "== and ToPrimitive rules are language-specific",
                "Emit explicit comparisons; preserve == null as nullish",
            ),
            h(
                "py-truthiness",
                "Python truthiness",
                &["python"],
                &["rust", "go"],
                "Empty containers are falsy",
                "Lower to explicit emptiness checks",
            ),
            h(
                "ruby-truthiness",
                "Ruby truthiness",
                &["ruby"],
                &["rust"],
                "Only false and nil are falsy",
                "Do not treat 0/\"\" as false",
            ),
            h(
                "string-indexing",
                "String indexing differences",
                &["python", "javascript"],
                &["rust"],
                "Code units vs scalars vs bytes",
                "Choose chars()/bytes() explicitly",
            ),
            h(
                "dict-order",
                "Dictionary ordering",
                &["python", "javascript"],
                &["rust"],
                "Insertion order guarantees vary by version/runtime",
                "Use IndexMap when order is observable",
            ),
            h(
                "regex-dialect",
                "Regex dialect",
                &["javascript", "python", "ruby"],
                &["rust", "go"],
                "Lookbehind / flags differ",
                "Flag unsupported patterns; do not silently alter",
            ),
            h(
                "dates-tz",
                "Date / timezone handling",
                &["javascript", "python"],
                &["rust", "go"],
                "Local vs UTC and DST edges",
                "Normalize to UTC instants + explicit TZ",
            ),
        ]
    }

    pub fn for_pair(&self, source: &str, target: &str) -> Vec<SemanticHazard> {
        self.builtins()
            .into_iter()
            .filter(|h| {
                h.source_langs.iter().any(|s| s == source)
                    && h.target_langs.iter().any(|t| t == target)
            })
            .collect()
    }
}

fn h(
    id: &str,
    title: &str,
    src: &[&str],
    tgt: &[&str],
    detail: &str,
    mitigation: &str,
) -> SemanticHazard {
    SemanticHazard {
        id: id.into(),
        title: title.into(),
        source_langs: src.iter().map(|s| (*s).to_string()).collect(),
        target_langs: tgt.iter().map(|s| (*s).to_string()).collect(),
        detail: detail.into(),
        mitigation: mitigation.into(),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StrategyOption {
    pub id: String,
    pub label: String,
    pub strategy: PreservationStrategy,
    pub behavior_confidence: f64,
    pub target_purity: f64,
    pub complexity: f64,
    pub notes: String,
}

#[derive(Clone, Debug, Default)]
pub struct StrategySearcher;

impl StrategySearcher {
    pub fn options_for_dependency(&self, name: &str) -> Vec<StrategyOption> {
        vec![
            StrategyOption {
                id: "equiv-crate".into(),
                label: format!("Equivalent target crate for {name}"),
                strategy: PreservationStrategy::Native,
                behavior_confidence: 0.85,
                target_purity: 1.0,
                complexity: 0.4,
                notes: "Prefer when API overlap is high".into(),
            },
            StrategyOption {
                id: "ffi-source".into(),
                label: format!("FFI / embed source runtime for {name}"),
                strategy: PreservationStrategy::Bridged,
                behavior_confidence: 0.95,
                target_purity: 0.2,
                complexity: 0.6,
                notes: "High fidelity, low purity".into(),
            },
            StrategyOption {
                id: "rewrite-used".into(),
                label: format!("Rewrite only used surface of {name}"),
                strategy: PreservationStrategy::BehaviorSynthesized,
                behavior_confidence: 0.7,
                target_purity: 0.9,
                complexity: 0.8,
                notes: "Requires behavioral contracts".into(),
            },
            StrategyOption {
                id: "reobserve".into(),
                label: format!("Reimplement from observed behavior of {name}"),
                strategy: PreservationStrategy::BehaviorSynthesized,
                behavior_confidence: 0.65,
                target_purity: 0.95,
                complexity: 0.9,
                notes: "Last resort when no mapping exists".into(),
            },
        ]
    }

    pub fn select(
        &self,
        options: &[StrategyOption],
        policy: &PreservationPolicy,
    ) -> Option<StrategyOption> {
        let scored = |o: &StrategyOption| -> f64 {
            match policy {
                PreservationPolicy::MaximumCompatibility => {
                    o.behavior_confidence * 0.7 + (1.0 - o.complexity) * 0.3
                }
                PreservationPolicy::MaximumNative => {
                    o.target_purity * 0.7 + o.behavior_confidence * 0.3
                }
                PreservationPolicy::MaximumPerformance => {
                    o.target_purity * 0.5 + o.behavior_confidence * 0.3 + (1.0 - o.complexity) * 0.2
                }
                PreservationPolicy::MinimumDependencies => {
                    o.target_purity * 0.6 + (1.0 - o.complexity) * 0.4
                }
                PreservationPolicy::FastestMigration => {
                    (1.0 - o.complexity) * 0.7 + o.behavior_confidence * 0.3
                }
            }
        };
        options
            .iter()
            .max_by(|a, b| {
                scored(a)
                    .partial_cmp(&scored(b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }
}
