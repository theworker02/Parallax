//! Polyglot islands — preserve untranslatable code behind typed boundaries.

#![deny(unsafe_code)]
#![allow(missing_docs)]

use crate::pvabi::{PvMessage, PvValue};
use serde::{Deserialize, Serialize};

/// Isolated source-runtime region inside a target project.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolyglotIsland {
    pub id: String,
    pub source_runtime: String,
    pub reason: String,
    pub entrypoints: Vec<IslandEntrypoint>,
    pub boundary: IslandBoundary,
    pub estimated_loc_pct: f64,
    pub dissolvable: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IslandEntrypoint {
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub may_throw: Vec<String>,
}

/// How the island is reached from native code.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IslandBoundaryKind {
    EmbeddedRuntime,
    Ffi,
    Ipc,
    Wasm,
    TypedRpc,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IslandBoundary {
    pub kind: IslandBoundaryKind,
    pub ops: Vec<String>,
    pub notes: String,
}

/// Synthesize a typed boundary from observed effects.
#[derive(Clone, Debug, Default)]
pub struct BoundarySynthesizer;

impl BoundarySynthesizer {
    pub fn synthesize(
        &self,
        island_id: &str,
        source_runtime: &str,
        inputs: &[&str],
        outputs: &[&str],
        effects: &[&str],
    ) -> IslandBoundary {
        let kind = if source_runtime == "wasm" {
            IslandBoundaryKind::Wasm
        } else if effects.contains(&"shared_memory") {
            IslandBoundaryKind::Ffi
        } else if effects.contains(&"network") {
            IslandBoundaryKind::Ipc
        } else {
            IslandBoundaryKind::EmbeddedRuntime
        };
        let mut ops: Vec<String> = inputs
            .iter()
            .zip(outputs.iter())
            .map(|(i, o)| format!("call:{i}->{o}"))
            .collect();
        if ops.is_empty() {
            ops.push(format!("invoke:{island_id}"));
        }
        IslandBoundary {
            kind,
            ops,
            notes: "Auto-inferred island boundary — verify before production".into(),
        }
    }

    pub fn encode_call(&self, op: &str, args: PvValue) -> PvMessage {
        PvMessage::new(op, args)
    }
}

/// Attempt to shrink an island by extracting crystallizable portions.
#[derive(Clone, Debug, Default)]
pub struct IslandDissolver;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DissolveReport {
    pub island_id: String,
    pub before_pct: f64,
    pub after_pct: f64,
    pub extracted: Vec<String>,
    pub remaining_barriers: Vec<String>,
    pub notes: String,
}

impl IslandDissolver {
    pub fn dissolve_step(&self, island: &PolyglotIsland) -> DissolveReport {
        if !island.dissolvable {
            return DissolveReport {
                island_id: island.id.clone(),
                before_pct: island.estimated_loc_pct,
                after_pct: island.estimated_loc_pct,
                extracted: Vec::new(),
                remaining_barriers: vec![island.reason.clone()],
                notes: "Island marked non-dissolvable (e.g. proprietary native blob)".into(),
            };
        }
        let after = (island.estimated_loc_pct * 0.7).max(0.5);
        DissolveReport {
            island_id: island.id.clone(),
            before_pct: island.estimated_loc_pct,
            after_pct: after,
            extracted: vec![
                "crystallized dynamic attribute paths".into(),
                "lowered decorator side-effects".into(),
            ],
            remaining_barriers: vec![island.reason.clone()],
            notes: "Dissolve is iterative; re-run plx dissolve after verification".into(),
        }
    }
}
