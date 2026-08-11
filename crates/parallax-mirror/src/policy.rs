//! Sync authority policies.

use serde::{Deserialize, Serialize};

/// Which side wins when both change.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SyncPolicy {
    /// Source is authoritative (default / safest).
    #[default]
    SourceAuthoritative,
    /// Target is authoritative.
    TargetAuthoritative,
    /// Attempt bidirectional with conflict reports.
    Bidirectional,
    /// Never auto-apply; report only.
    Manual,
}

impl SyncPolicy {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "source-authoritative" | "source" => Some(Self::SourceAuthoritative),
            "target-authoritative" | "target" => Some(Self::TargetAuthoritative),
            "bidirectional" | "bi" => Some(Self::Bidirectional),
            "manual" => Some(Self::Manual),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::SourceAuthoritative => "source-authoritative",
            Self::TargetAuthoritative => "target-authoritative",
            Self::Bidirectional => "bidirectional",
            Self::Manual => "manual",
        }
    }
}

/// Alias used in link.json.
pub type LinkPolicy = SyncPolicy;
