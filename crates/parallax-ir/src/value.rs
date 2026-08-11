//! PIR value types.

use crate::Result;
use indexmap::IndexMap;
use parallax_core::{ErrorCode, ObjectId, ParallaxError, SemanticLoss};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Arbitrary-precision integer stored as decimal text plus optional signed i128 cache.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PirInteger {
    /// Decimal representation (allows values beyond i128).
    pub decimal: String,
}

impl PirInteger {
    /// Construct from i64.
    pub fn from_i64(v: i64) -> Self {
        Self {
            decimal: v.to_string(),
        }
    }

    /// Construct from i128.
    pub fn from_i128(v: i128) -> Self {
        Self {
            decimal: v.to_string(),
        }
    }

    /// Parse from decimal string.
    pub fn from_decimal(s: impl Into<String>) -> Self {
        Self { decimal: s.into() }
    }

    /// Try parse as i128.
    pub fn as_i128(&self) -> Option<i128> {
        self.decimal.parse().ok()
    }

    /// Semantic loss when targeting JavaScript Number.
    pub fn js_number_loss(&self) -> SemanticLoss {
        match self.as_i128() {
            Some(v) => parallax_core::integer_to_js_number_loss(v),
            None => SemanticLoss::Lossy,
        }
    }
}

impl fmt::Display for PirInteger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.decimal)
    }
}

/// Map entry preserving key identity (string keys common; PIR allows any value key).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PirMapEntry {
    /// Map key.
    pub key: PirValue,
    /// Map value.
    pub value: PirValue,
}

/// Language-neutral PIR value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum PirValue {
    /// JSON/Python/JS null / None / undefined(as null).
    Null,
    /// Boolean.
    Bool {
        /// Value.
        v: bool,
    },
    /// Integer (arbitrary precision decimal).
    Int {
        /// Integer payload.
        v: PirInteger,
    },
    /// IEEE-754 binary64 float.
    Float {
        /// Float value. Non-finite values use string tokens on the wire (`NaN`, `Infinity`, `-Infinity`).
        #[serde(with = "float_wire")]
        v: f64,
    },
    /// UTF-8 string.
    String {
        /// String payload.
        v: String,
    },
    /// Raw bytes (base64 on the wire via helper methods).
    Bytes {
        /// Raw bytes.
        #[serde(with = "bytes_b64")]
        v: Vec<u8>,
    },
    /// Ordered list / array.
    List {
        /// Elements.
        v: Vec<PirValue>,
    },
    /// Ordered tuple (semantic distinction from list).
    Tuple {
        /// Elements.
        v: Vec<PirValue>,
    },
    /// Set (order not significant; stored ordered for determinism).
    Set {
        /// Elements.
        v: Vec<PirValue>,
    },
    /// Ordered map / object / dict.
    Map {
        /// Entries in insertion order.
        entries: Vec<PirMapEntry>,
    },
    /// Explicit BigInt (distinct from Int for JS restore).
    #[serde(rename = "bigint")]
    BigInt {
        /// Decimal digits.
        v: String,
    },
    /// Function / callable reference — not migratable by default.
    Function {
        /// Optional name.
        name: Option<String>,
        /// Language-specific descriptor.
        descriptor: String,
    },
    /// Graph reference to another object in the document.
    Ref {
        /// Target object id.
        id: ObjectId,
    },
    /// Explicit unsupported payload — never silently dropped.
    Unsupported {
        /// Why it cannot be represented.
        reason: String,
        /// Best-effort debug representation from the source runtime.
        repr: String,
        /// Source type name if known.
        type_name: Option<String>,
    },
}

impl PirValue {
    /// Convenience bool constructor.
    pub fn bool(v: bool) -> Self {
        Self::Bool { v }
    }

    /// Convenience int constructor.
    pub fn int_i64(v: i64) -> Self {
        Self::Int {
            v: PirInteger::from_i64(v),
        }
    }

    /// Convenience string constructor.
    pub fn string(s: impl Into<String>) -> Self {
        Self::String { v: s.into() }
    }

    /// Convenience list constructor.
    pub fn list(v: Vec<PirValue>) -> Self {
        Self::List { v }
    }

    /// Build a string-keyed map from an IndexMap.
    pub fn string_map(map: IndexMap<String, PirValue>) -> Self {
        Self::Map {
            entries: map
                .into_iter()
                .map(|(k, v)| PirMapEntry {
                    key: PirValue::string(k),
                    value: v,
                })
                .collect(),
        }
    }

    /// If this is a string-keyed map, return IndexMap view.
    pub fn as_string_map(&self) -> Option<IndexMap<String, PirValue>> {
        match self {
            Self::Map { entries } => {
                let mut out = IndexMap::new();
                for e in entries {
                    match &e.key {
                        Self::String { v: k } => {
                            out.insert(k.clone(), e.value.clone());
                        }
                        _ => return None,
                    }
                }
                Some(out)
            }
            _ => None,
        }
    }

    /// Short type label for diagnostics.
    pub fn type_label(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Bool { .. } => "bool",
            Self::Int { .. } => "int",
            Self::Float { .. } => "float",
            Self::String { .. } => "string",
            Self::Bytes { .. } => "bytes",
            Self::List { .. } => "list",
            Self::Tuple { .. } => "tuple",
            Self::Set { .. } => "set",
            Self::Map { .. } => "map",
            Self::BigInt { .. } => "bigint",
            Self::Function { .. } => "function",
            Self::Ref { .. } => "ref",
            Self::Unsupported { .. } => "unsupported",
        }
    }

    /// Convert to tagged JSON (same as serde for most variants; float special-cased).
    ///
    /// Returns `Unsupported` JSON on rare serialization failure instead of silently
    /// collapsing to JSON `null` (which would be indistinguishable from `PirValue::Null`).
    pub fn to_json(&self) -> serde_json::Value {
        match serde_json::to_value(self) {
            Ok(v) => v,
            Err(e) => serde_json::json!({
                "t": "unsupported",
                "reason": "serialization_failure",
                "repr": e.to_string(),
                "type_name": self.type_label(),
            }),
        }
    }

    /// Parse from tagged JSON value.
    pub fn from_json(v: &serde_json::Value) -> Result<Self> {
        serde_json::from_value(v.clone()).map_err(|e| {
            ParallaxError::new(ErrorCode::SerializationFailure, e.to_string())
                .with_source("parallax-ir")
                .with_operation("PirValue::from_json")
        })
    }

    /// Walk the value tree, invoking `f` on each node (pre-order).
    pub fn walk<F: FnMut(&PirValue)>(&self, f: &mut F) {
        f(self);
        match self {
            Self::List { v: xs } | Self::Tuple { v: xs } | Self::Set { v: xs } => {
                for x in xs {
                    x.walk(f);
                }
            }
            Self::Map { entries } => {
                for e in entries {
                    e.key.walk(f);
                    e.value.walk(f);
                }
            }
            _ => {}
        }
    }

    /// Summarize for CLI display.
    pub fn summary(&self) -> String {
        match self {
            Self::Null => "null".into(),
            Self::Bool { v } => v.to_string(),
            Self::Int { v } => v.decimal.clone(),
            Self::Float { v } => format!("{v}"),
            Self::String { v: s } => {
                let mut chars = s.chars();
                let head: String = chars.by_ref().take(45).collect();
                if chars.next().is_some() {
                    format!("\"{head}…\"")
                } else {
                    format!("\"{s}\"")
                }
            }
            Self::Bytes { v } => format!("bytes[{}]", v.len()),
            Self::List { v: xs } => format!("list[{}]", xs.len()),
            Self::Tuple { v: xs } => format!("tuple[{}]", xs.len()),
            Self::Set { v: xs } => format!("set[{}]", xs.len()),
            Self::Map { entries } => format!("map{{{}}}", entries.len()),
            Self::BigInt { v } => format!("bigint({v})"),
            Self::Function { name, .. } => {
                format!("function({})", name.as_deref().unwrap_or("?"))
            }
            Self::Ref { id } => format!("ref{id}"),
            Self::Unsupported { reason, .. } => format!("unsupported({reason})"),
        }
    }
}

/// Wire encoding for f64 that preserves NaN/±Infinity (serde_json rejects non-finite numbers).
mod float_wire {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(v: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if v.is_nan() {
            "NaN".serialize(serializer)
        } else if *v == f64::INFINITY {
            "Infinity".serialize(serializer)
        } else if *v == f64::NEG_INFINITY {
            "-Infinity".serialize(serializer)
        } else {
            v.serialize(serializer)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum FloatWire {
            Num(f64),
            Text(String),
        }
        match FloatWire::deserialize(deserializer)? {
            FloatWire::Num(n) => Ok(n),
            FloatWire::Text(s) => match s.as_str() {
                "NaN" => Ok(f64::NAN),
                "Infinity" | "+Infinity" => Ok(f64::INFINITY),
                "-Infinity" => Ok(f64::NEG_INFINITY),
                other => Err(serde::de::Error::custom(format!(
                    "invalid float token: {other}"
                ))),
            },
        }
    }
}

mod bytes_b64 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&b64_encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        b64_decode(&s).map_err(serde::de::Error::custom)
    }

    fn b64_encode(bytes: &[u8]) -> String {
        const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for chunk in bytes.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
            let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
            let n = (b0 << 16) | (b1 << 8) | b2;
            out.push(TABLE[((n >> 18) & 63) as usize] as char);
            out.push(TABLE[((n >> 12) & 63) as usize] as char);
            out.push(if chunk.len() > 1 {
                TABLE[((n >> 6) & 63) as usize] as char
            } else {
                '='
            });
            out.push(if chunk.len() > 2 {
                TABLE[(n & 63) as usize] as char
            } else {
                '='
            });
        }
        out
    }

    fn b64_decode(input: &str) -> Result<Vec<u8>, String> {
        fn val(c: u8) -> Result<u8, String> {
            match c {
                b'A'..=b'Z' => Ok(c - b'A'),
                b'a'..=b'z' => Ok(c - b'a' + 26),
                b'0'..=b'9' => Ok(c - b'0' + 52),
                b'+' => Ok(62),
                b'/' => Ok(63),
                _ => Err(format!("invalid base64 byte: {c}")),
            }
        }
        let bytes = input.as_bytes();
        if !bytes.is_empty() && bytes.len() % 4 != 0 {
            return Err("invalid base64 length".into());
        }
        let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
        for chunk in bytes.chunks(4) {
            if chunk.len() < 4 {
                return Err("invalid base64 chunk".into());
            }
            let n = ((val(chunk[0])? as u32) << 18)
                | ((val(chunk[1])? as u32) << 12)
                | ((if chunk[2] == b'=' {
                    0
                } else {
                    val(chunk[2])? as u32
                }) << 6)
                | (if chunk[3] == b'=' {
                    0
                } else {
                    val(chunk[3])? as u32
                });
            out.push(((n >> 16) & 0xff) as u8);
            if chunk[2] != b'=' {
                out.push(((n >> 8) & 0xff) as u8);
            }
            if chunk[3] != b'=' {
                out.push((n & 0xff) as u8);
            }
        }
        Ok(out)
    }
}
