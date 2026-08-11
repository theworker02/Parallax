//! Concurrency Intent Representation (CIR).

#![deny(unsafe_code)]
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConcurrencyIntent {
    Parallel,
    Concurrent,
    MessagePassing,
    SharedOwnership,
    Join,
    Race,
    Timeout,
    Cancellation,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConcurrencyGraph {
    pub tasks: Vec<String>,
    pub channels: Vec<String>,
    pub intents: Vec<ConcurrencyIntent>,
    pub notes: String,
}

impl ConcurrencyGraph {
    pub fn from_signals(signals: &[&str]) -> Self {
        let mut intents = Vec::new();
        for s in signals {
            match *s {
                "async/promises" | "asyncio" => intents.push(ConcurrencyIntent::Concurrent),
                "threads" => intents.push(ConcurrencyIntent::Parallel),
                "channels" => intents.push(ConcurrencyIntent::MessagePassing),
                "timeout" => intents.push(ConcurrencyIntent::Timeout),
                "cancel" => intents.push(ConcurrencyIntent::Cancellation),
                _ => {}
            }
        }
        Self {
            tasks: Vec::new(),
            channels: Vec::new(),
            intents,
            notes: "CIR captures intent; target adapter chooses Tokio/goroutines/etc.".into(),
        }
    }
}
