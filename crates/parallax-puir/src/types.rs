//! PUIR type system and inference evidence.

use crate::Confidence;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Language-neutral type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PuirType {
    /// Unknown / uninferred.
    Unknown,
    /// Unit / void / undefined return.
    Unit,
    /// Boolean.
    Bool,
    /// Signed integer (preferred width hint).
    Int {
        /// Bits if known (32/64).
        bits: Option<u32>,
    },
    /// Floating point.
    Float {
        /// Bits if known (32/64).
        bits: Option<u32>,
    },
    /// UTF-8 string.
    String,
    /// Bytes.
    Bytes,
    /// Nullable / optional wrapper.
    Optional {
        /// Inner type.
        inner: Box<PuirType>,
    },
    /// List / Vec / array.
    List {
        /// Element type.
        element: Box<PuirType>,
    },
    /// Map / Record / HashMap.
    Map {
        /// Key type.
        key: Box<PuirType>,
        /// Value type.
        value: Box<PuirType>,
    },
    /// Named nominal type (class/interface/struct).
    Named {
        /// Type name.
        name: String,
        /// Optional module path.
        module: Option<String>,
    },
    /// Future / Promise.
    Future {
        /// Output type.
        output: Box<PuirType>,
    },
    /// Result / fallible.
    Result {
        /// Ok type.
        ok: Box<PuirType>,
        /// Err type name hint.
        err: Box<PuirType>,
    },
    /// Function type.
    Function {
        /// Parameters.
        params: Vec<PuirType>,
        /// Return type.
        ret: Box<PuirType>,
        /// Async.
        async_: bool,
    },
    /// Union of alternatives.
    Union {
        /// Members.
        members: Vec<PuirType>,
    },
    /// Explicitly unsupported source type.
    Unsupported {
        /// Original spelling.
        original: String,
    },
}

impl PuirType {
    /// Convenience int64.
    pub fn i64() -> Self {
        Self::Int { bits: Some(64) }
    }

    /// Convenience f64.
    pub fn f64() -> Self {
        Self::Float { bits: Some(64) }
    }
}

/// One piece of evidence used during type inference.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TypeEvidence {
    /// Variable or symbol name.
    pub symbol: String,
    /// Kind of evidence.
    pub kind: String,
    /// Detail.
    pub detail: String,
    /// Source file if known.
    pub file: Option<String>,
    /// Source line if known.
    pub line: Option<u32>,
}

/// Inference outcome for a symbol.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TypeInferenceReport {
    /// Symbol name.
    pub symbol: String,
    /// Inferred type.
    pub inferred: PuirType,
    /// Confidence.
    pub confidence: Confidence,
    /// Evidence list.
    pub evidence: Vec<TypeEvidence>,
    /// Alternatives when ambiguous.
    pub alternatives: Vec<PuirType>,
    /// Whether manual review is required.
    pub manual_review: bool,
}

/// Collection of inference reports.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TypeInferenceBundle {
    /// Reports by symbol.
    pub reports: IndexMap<String, TypeInferenceReport>,
}
