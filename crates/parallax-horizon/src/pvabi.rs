//! Parallax Value ABI (PVABI) — stable values across language boundaries.

#![allow(missing_docs)]

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// PVABI schema version (independent of product SemVer).
pub const PVABI_SCHEMA_VERSION: u32 = 1;

/// Stable cross-language value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum PvValue {
    Null,
    Bool(bool),
    I64(i64),
    U64(u64),
    F64(f64),
    String(String),
    Bytes(Vec<u8>),
    List(Vec<PvValue>),
    Map(IndexMap<String, PvValue>),
    Record {
        name: Option<String>,
        fields: IndexMap<String, PvValue>,
    },
    Enum {
        name: String,
        variant: String,
        #[serde(default)]
        payload: Option<Box<PvValue>>,
    },
    Result {
        ok: bool,
        #[serde(default)]
        value: Option<Box<PvValue>>,
        #[serde(default)]
        error: Option<Box<PvValue>>,
    },
    /// Opaque host handle (island / FFI / WASM).
    Handle {
        runtime: String,
        id: u64,
    },
    /// Explicit reference identity (shared state across boundary).
    Reference {
        id: String,
    },
}

impl PvValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

/// Typed boundary message using PVABI.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PvMessage {
    pub schema: u32,
    pub op: String,
    pub payload: PvValue,
}

impl PvMessage {
    pub fn new(op: impl Into<String>, payload: PvValue) -> Self {
        Self {
            schema: PVABI_SCHEMA_VERSION,
            op: op.into(),
            payload,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_map() {
        let mut m = IndexMap::new();
        m.insert("a".into(), PvValue::I64(1));
        let v = PvValue::Map(m);
        let s = serde_json::to_string(&v).unwrap();
        let back: PvValue = serde_json::from_str(&s).unwrap();
        assert_eq!(v, back);
    }
}
