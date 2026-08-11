//! Parallax Universal Intermediate Representation (PUIR).
//!
//! PUIR represents **language-neutral program semantics** (intent), not syntax.
//! Distinct from PIR (values), PCIR (suspended control), and UES (execution state).

#![deny(unsafe_code)]
#![warn(missing_docs)]

mod expr;
mod item;
mod stmt;
mod types;
mod version;

pub use expr::Expr;
pub use item::{Function, Module, Parameter, PuirItem, TypeDef, Visibility};
pub use stmt::Stmt;
pub use types::{PuirType, TypeEvidence, TypeInferenceBundle, TypeInferenceReport};
pub use version::{check_puir_schema, PUIR_SCHEMA_VERSION};

use indexmap::IndexMap;
use parallax_core::{ErrorCode, ParallaxError};
use serde::{Deserialize, Serialize};

/// Source location preserved for diagnostics and `.plxmap` origin lookup.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    /// Path relative to project root.
    pub file: String,
    /// 1-based start line.
    pub line: u32,
    /// 1-based start column.
    pub column: u32,
    /// Optional end line.
    pub end_line: Option<u32>,
    /// Optional end column.
    pub end_column: Option<u32>,
}

impl SourceSpan {
    /// Construct a point span.
    pub fn new(file: impl Into<String>, line: u32, column: u32) -> Self {
        Self {
            file: file.into(),
            line,
            column,
            end_line: None,
            end_column: None,
        }
    }
}

/// Confidence attached to a translated region.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    /// Bit-for-bit or proven-equivalent semantics.
    Exact,
    /// High confidence idiomatic translation.
    High,
    /// Plausible but not fully verified.
    Medium,
    /// Ambiguous; needs review.
    Low,
    /// Cannot be represented.
    Unsupported,
}

impl Confidence {
    /// Numeric score in `[0.0, 1.0]` for aggregation (Unsupported = 0).
    pub fn score(self) -> f64 {
        match self {
            Self::Exact => 1.0,
            Self::High => 0.95,
            Self::Medium => 0.75,
            Self::Low => 0.45,
            Self::Unsupported => 0.0,
        }
    }

    /// Compact label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Exact => "EXACT",
            Self::High => "HIGH",
            Self::Medium => "MEDIUM",
            Self::Low => "LOW",
            Self::Unsupported => "UNSUPPORTED",
        }
    }
}

/// A complete PUIR program (one or more modules).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PuirProgram {
    /// Schema version.
    pub version: u32,
    /// Modules keyed by stable module id / path.
    pub modules: IndexMap<String, Module>,
    /// Free-form metadata.
    pub metadata: IndexMap<String, serde_json::Value>,
}

impl PuirProgram {
    /// Empty program at current schema version.
    pub fn new() -> Self {
        Self {
            version: PUIR_SCHEMA_VERSION,
            modules: IndexMap::new(),
            metadata: IndexMap::new(),
        }
    }

    /// Validate schema version.
    pub fn validate(&self) -> Result<(), ParallaxError> {
        check_puir_schema(self.version)
    }
}

impl Default for PuirProgram {
    fn default() -> Self {
        Self::new()
    }
}

/// Stable node id within a PUIR graph (for source maps / repair).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(transparent)]
pub struct NodeId(pub u64);

impl NodeId {
    /// Create from raw.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Effect flags for functions / regions.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Effects {
    /// May perform I/O.
    pub io: bool,
    /// May await async operations.
    pub async_: bool,
    /// May throw / return error.
    pub throws: bool,
    /// May read environment / config.
    pub env: bool,
    /// May access filesystem.
    pub fs: bool,
    /// May perform network.
    pub network: bool,
}

/// Helper to build a schema error.
pub fn schema_error(msg: impl Into<String>) -> ParallaxError {
    ParallaxError::new(ErrorCode::InvalidArgument, msg)
        .with_source("parallax-puir")
        .with_operation("validate")
}
