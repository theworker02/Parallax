//! Behavioral contracts — observed software behavior as migration truth.

#![deny(unsafe_code)]
#![allow(missing_docs)]

use crate::pvabi::PvValue;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BehavioralContract {
    pub function: String,
    pub input_type: String,
    pub observations: Vec<BehaviorObservation>,
    pub confidence: f64,
    pub sources: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BehaviorObservation {
    pub when: String,
    pub then: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_input: Option<PvValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_output: Option<PvValue>,
}

impl BehavioralContract {
    pub fn summarize(&self) -> String {
        let mut lines = vec![format!("Function: {}", self.function)];
        lines.push(format!("Input: {}", self.input_type));
        lines.push("Observed behavior:".into());
        for o in &self.observations {
            lines.push(format!("  {} → {}", o.when, o.then));
        }
        lines.push(format!("Confidence: {:.0}%", self.confidence * 100.0));
        lines.join("\n")
    }
}
