//! Parallax Intermediate Representation (PIR).
//!
//! Language-neutral value graphs used for capture, snapshot, and migration.

#![deny(unsafe_code)]
#![warn(missing_docs)]

mod document;
mod hash;
mod value;

pub use document::{PirBinding, PirDocument, PirRoot};
pub use hash::content_hash;
pub use value::{PirInteger, PirMapEntry, PirValue};

use parallax_core::{ErrorCode, ParallaxError, PIR_SCHEMA_VERSION};

/// Current PIR schema version re-export.
pub const SCHEMA_VERSION: u32 = PIR_SCHEMA_VERSION;

/// Result alias for IR operations.
pub type Result<T> = parallax_core::Result<T>;

/// Serialize a PIR document to pretty JSON bytes.
pub fn to_json_bytes(doc: &PirDocument) -> Result<Vec<u8>> {
    serde_json::to_vec_pretty(doc).map_err(|e| {
        ParallaxError::new(ErrorCode::SerializationFailure, e.to_string())
            .with_source("parallax-ir")
            .with_operation("to_json_bytes")
    })
}

/// Serialize a PIR document to compact JSON bytes.
pub fn to_json_bytes_compact(doc: &PirDocument) -> Result<Vec<u8>> {
    serde_json::to_vec(doc).map_err(|e| {
        ParallaxError::new(ErrorCode::SerializationFailure, e.to_string())
            .with_source("parallax-ir")
            .with_operation("to_json_bytes_compact")
    })
}

/// Deserialize a PIR document from JSON bytes.
pub fn from_json_bytes(bytes: &[u8]) -> Result<PirDocument> {
    let doc: PirDocument = serde_json::from_slice(bytes).map_err(|e| {
        ParallaxError::new(ErrorCode::SerializationFailure, e.to_string())
            .with_source("parallax-ir")
            .with_operation("from_json_bytes")
    })?;
    doc.validate()?;
    Ok(doc)
}

/// Parse a PIR value from a JSON value using the tagged wire format.
pub fn value_from_json(v: &serde_json::Value) -> Result<PirValue> {
    PirValue::from_json(v)
}

/// Convert a PIR value to the tagged JSON wire format.
pub fn value_to_json(v: &PirValue) -> serde_json::Value {
    v.to_json()
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;
    use parallax_core::{ErrorCode, ObjectId};

    #[test]
    fn rejects_future_schema_on_deserialize() {
        let raw = br#"{"schema":99,"bindings":{},"objects":{},"roots":[],"metadata":{}}"#;
        let err = from_json_bytes(raw).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSnapshot);
        assert!(err.message.contains("unsupported PIR schema"));
    }

    #[test]
    fn rejects_zero_schema() {
        let mut doc = PirDocument::new();
        doc.schema = 0;
        assert!(doc.validate().is_err());
    }

    #[test]
    fn round_trip_demo_state() {
        let mut fields: IndexMap<String, PirValue> = IndexMap::new();
        fields.insert("username".into(), PirValue::string("Ada"));
        fields.insert("score".into(), PirValue::int_i64(42));
        fields.insert(
            "projects".into(),
            PirValue::list(vec![
                PirValue::string("compiler"),
                PirValue::string("runtime"),
                PirValue::string("vm"),
            ]),
        );
        let state = PirValue::Map {
            entries: fields
                .into_iter()
                .map(|(k, v)| PirMapEntry {
                    key: PirValue::string(k),
                    value: v,
                })
                .collect(),
        };
        let mut doc = PirDocument::new();
        doc.set_binding("state", state);
        let bytes = to_json_bytes(&doc).unwrap();
        let restored = from_json_bytes(&bytes).unwrap();
        assert_eq!(doc.bindings.len(), restored.bindings.len());
        let s = restored.binding("state").unwrap();
        match s {
            PirValue::Map { entries } => {
                assert_eq!(entries.len(), 3);
            }
            other => panic!("expected map, got {other:?}"),
        }
    }

    #[test]
    fn non_finite_float_round_trips_without_becoming_null() {
        for (v, token) in [
            (f64::NAN, "NaN"),
            (f64::INFINITY, "Infinity"),
            (f64::NEG_INFINITY, "-Infinity"),
        ] {
            let pir = PirValue::Float { v };
            let json = pir.to_json();
            assert_eq!(json["t"], "float");
            assert_eq!(json["v"], token);
            let restored = PirValue::from_json(&json).unwrap();
            match restored {
                PirValue::Float { v: got } => {
                    if v.is_nan() {
                        assert!(got.is_nan());
                    } else {
                        assert_eq!(got, v);
                    }
                }
                other => panic!("expected float, got {other:?}"),
            }
        }
    }

    #[test]
    fn summary_handles_multibyte_prefix_without_panic() {
        // Multi-byte UTF-8 beyond 45 bytes previously panicked on &s[..45].
        let s = "café".repeat(20);
        let summary = PirValue::string(s).summary();
        assert!(summary.starts_with('"'));
        assert!(summary.ends_with('"') || summary.contains('…'));
    }

    #[test]
    fn validate_rejects_dangling_refs() {
        let mut doc = PirDocument::new();
        doc.set_binding(
            "x",
            PirValue::Ref {
                id: ObjectId::new(99),
            },
        );
        let err = doc.validate().unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSnapshot);
    }
}
