//! Counterexample-guided repair (CEGIS-style loop).

#![deny(unsafe_code)]
#![allow(missing_docs)]

use crate::pvabi::PvValue;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Counterexample {
    pub input: PvValue,
    pub source_output: PvValue,
    pub target_output: PvValue,
    pub note: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RepairStep {
    pub iteration: u32,
    pub counterexample: Counterexample,
    pub patch_hint: String,
}

#[derive(Clone, Debug, Default)]
pub struct CegisLoop;

impl CegisLoop {
    pub fn next_repair(&self, iteration: u32, cex: Counterexample) -> RepairStep {
        let patch_hint = format!(
            "Align target with source for input {:?}: expected {:?}, got {:?}",
            cex.input, cex.source_output, cex.target_output
        );
        RepairStep {
            iteration,
            counterexample: cex,
            patch_hint,
        }
    }
}
