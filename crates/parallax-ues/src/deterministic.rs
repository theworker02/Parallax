//! Deterministic replay hooks and journal schema.
//!
//! The replay **engine** is not implemented. Types and status markers exist so
//! Continuum never pretends replay succeeded.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Whether a deterministic replay engine can consume this context.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayEngineStatus {
    /// Engine ready for this context.
    Ready,
    /// Partial / experimental.
    Experimental,
    /// Not implemented — attempts must return Unsupported.
    Unsupported,
}

/// One journaled non-deterministic event (schema only).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReplayJournalEntry {
    /// Monotonic sequence number.
    pub seq: u64,
    /// Event kind (`rng`, `clock`, `io`, `syscall`, …).
    pub kind: String,
    /// Opaque payload.
    #[serde(default)]
    pub payload: serde_json::Value,
}

/// Append-only replay journal schema.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct ReplayJournal {
    /// Journal schema version (independent; start at 1).
    pub schema: u32,
    /// Ordered entries.
    #[serde(default)]
    pub entries: Vec<ReplayJournalEntry>,
}

impl ReplayJournal {
    /// Empty journal at schema 1.
    pub fn new() -> Self {
        Self {
            schema: 1,
            entries: Vec::new(),
        }
    }
}

/// Deterministic context attached to a UES.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeterministicContext {
    /// Engine readiness — typically Unsupported until a real engine ships.
    pub engine_status: ReplayEngineStatus,
    /// Seed when recording was attempted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    /// Optional journal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub journal: Option<ReplayJournal>,
    /// Human-readable reason when unsupported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unsupported_reason: Option<String>,
    /// Extension bag for future recorder fields.
    #[serde(default)]
    pub extensions: IndexMap<String, serde_json::Value>,
}

impl DeterministicContext {
    /// Honest default: replay engine not ready.
    pub fn unsupported_engine() -> Self {
        Self {
            engine_status: ReplayEngineStatus::Unsupported,
            seed: None,
            journal: None,
            unsupported_reason: Some(
                "Deterministic replay engine is not implemented; journal schema only".into(),
            ),
            extensions: IndexMap::new(),
        }
    }

    /// Attempt to begin replay — always Unsupported until an engine exists.
    pub fn begin_replay(&self) -> parallax_core::Result<()> {
        match self.engine_status {
            ReplayEngineStatus::Ready => Ok(()),
            ReplayEngineStatus::Experimental => Err(parallax_core::ParallaxError::new(
                parallax_core::ErrorCode::CapabilityViolation,
                "deterministic replay is experimental and not enabled for this UES",
            )
            .with_source("parallax-ues")
            .with_operation("begin_replay")
            .context("engine_status", "experimental")),
            ReplayEngineStatus::Unsupported => Err(parallax_core::ParallaxError::new(
                parallax_core::ErrorCode::UnsupportedValue,
                self.unsupported_reason
                    .clone()
                    .unwrap_or_else(|| "deterministic replay Unsupported".into()),
            )
            .with_source("parallax-ues")
            .with_operation("begin_replay")
            .context("engine_status", "unsupported")),
        }
    }
}

impl Default for DeterministicContext {
    fn default() -> Self {
        Self::unsupported_engine()
    }
}
