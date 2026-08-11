//! Three-way semantic merge.

use crate::identity::SemanticId;
use serde::{Deserialize, Serialize};

/// Conflict when base/source/target diverge.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SemanticConflict {
    pub id: SemanticId,
    pub kind: String,
    pub qualified_name: String,
    pub base: String,
    pub source: String,
    pub target: String,
}

/// Merge decision — never guess on conflicts.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeDecision {
    TakeSource,
    TakeTarget,
    TakeBase,
    Conflict(SemanticConflict),
}

/// Result of a three-way merge analysis.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SemanticMerge {
    pub decisions: Vec<MergeDecision>,
    pub conflicts: Vec<SemanticConflict>,
}

impl SemanticMerge {
    /// Compare constant-like string values for a node.
    pub fn constant_conflict(
        id: SemanticId,
        name: &str,
        base: &str,
        source: &str,
        target: &str,
    ) -> Option<SemanticConflict> {
        if base != source && base != target && source != target {
            Some(SemanticConflict {
                id,
                kind: "ChangedConstant".into(),
                qualified_name: name.into(),
                base: base.into(),
                source: source.into(),
                target: target.into(),
            })
        } else {
            None
        }
    }
}
