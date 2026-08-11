//! Generated vs manual region ownership.

use crate::identity::SemanticId;
use serde::{Deserialize, Serialize};

/// Ownership of a code region.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegionKind {
    Generated,
    Manual,
    Shared,
}

/// Sidecar ownership record (preferred over inline markers).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegionOwnership {
    pub id: SemanticId,
    pub kind: RegionKind,
    pub target_file: String,
    /// Content hash of last generated body (for manual-edit detection).
    pub content_hash: String,
    pub reverse_safe: ReverseSafety,
}

/// Whether reverse sync is allowed for this node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReverseSafety {
    ExactYes,
    IdiomaticPartial,
    FrameworkSpecificNo,
    ManualRegionNo,
}

/// Manual region entry stored in `manual-regions.json`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManualRegion {
    pub id: SemanticId,
    pub target_file: String,
    pub classification: ManualClassification,
    pub note: String,
}

/// Classification of a target-side edit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManualClassification {
    FormattingOnly,
    EquivalentRefactor,
    BehaviorChange,
    Unknown,
}
