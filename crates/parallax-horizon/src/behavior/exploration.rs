//! Behavior exploration, triangulation, and repair coordination.

#![deny(unsafe_code)]
#![allow(missing_docs)]

use super::contracts::{BehaviorObservation, BehavioralContract};
use crate::pvabi::PvValue;
use serde::{Deserialize, Serialize};

/// Black-box exploration of a function via structured inputs.
#[derive(Clone, Debug, Default)]
pub struct BehaviorExplorer;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExplorationPlan {
    pub function: String,
    pub inputs: Vec<PvValue>,
    pub capture: Vec<String>,
}

impl BehaviorExplorer {
    pub fn plan_default_inputs(&self, function: &str) -> ExplorationPlan {
        ExplorationPlan {
            function: function.into(),
            inputs: vec![
                PvValue::Null,
                PvValue::String(String::new()),
                PvValue::String("  PADDED  ".into()),
                PvValue::String("UPPER@EXAMPLE.COM".into()),
                PvValue::F64(f64::NAN),
                PvValue::F64(-0.0),
                PvValue::I64(0),
                PvValue::List(vec![]),
            ],
            capture: vec!["return".into(), "exceptions".into(), "mutations".into()],
        }
    }

    pub fn contract_from_observations(
        &self,
        function: &str,
        input_type: &str,
        observations: Vec<BehaviorObservation>,
    ) -> BehavioralContract {
        let n = observations.len().max(1) as f64;
        BehavioralContract {
            function: function.into(),
            input_type: input_type.into(),
            observations,
            confidence: (0.7_f64 + 0.05 * n).min(0.98_f64),
            sources: vec!["behavior_explorer".into()],
        }
    }
}

/// Combine static / runtime / test signals into a confidence score.
#[derive(Clone, Debug, Default)]
pub struct SemanticTriangulator;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TriangulationReport {
    pub static_confidence: f64,
    pub runtime_confidence: f64,
    pub tests_confidence: f64,
    pub combined: f64,
    pub notes: String,
}

impl SemanticTriangulator {
    pub fn combine(&self, static_c: f64, runtime_c: f64, tests_c: f64) -> TriangulationReport {
        let combined = static_c * 0.2 + runtime_c * 0.35 + tests_c * 0.45;
        TriangulationReport {
            static_confidence: static_c,
            runtime_confidence: runtime_c,
            tests_confidence: tests_c,
            combined,
            notes: "Combined semantic confidence — not a formal proof".into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SemanticFuzzCase {
    pub name: String,
    pub value: PvValue,
    pub hazard: String,
}

#[derive(Clone, Debug, Default)]
pub struct SemanticFuzzer;

impl SemanticFuzzer {
    pub fn cross_language_hazards(&self) -> Vec<SemanticFuzzCase> {
        vec![
            SemanticFuzzCase {
                name: "nan".into(),
                value: PvValue::F64(f64::NAN),
                hazard: "NaN inequality / JSON encoding".into(),
            },
            SemanticFuzzCase {
                name: "neg_zero".into(),
                value: PvValue::F64(-0.0),
                hazard: "signed zero formatting".into(),
            },
            SemanticFuzzCase {
                name: "infinity".into(),
                value: PvValue::F64(f64::INFINITY),
                hazard: "Infinity handling".into(),
            },
            SemanticFuzzCase {
                name: "empty_string".into(),
                value: PvValue::String(String::new()),
                hazard: "truthiness / empty".into(),
            },
            SemanticFuzzCase {
                name: "null".into(),
                value: PvValue::Null,
                hazard: "null vs undefined vs Option".into(),
            },
            SemanticFuzzCase {
                name: "big_int".into(),
                value: PvValue::String("9007199254740993".into()),
                hazard: "JS Number precision boundary".into(),
            },
            SemanticFuzzCase {
                name: "unicode_nfc".into(),
                value: PvValue::String("é".into()),
                hazard: "unicode normalization".into(),
            },
        ]
    }
}
