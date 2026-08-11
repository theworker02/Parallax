//! Value conversion between runtime conventions inside PIR.

use parallax_core::{ConversionPolicy, ErrorCode, ParallaxError, RuntimeKind, SemanticLoss};
use parallax_ir::{PirDocument, PirInteger, PirMapEntry, PirValue};

/// Convert a document for a target runtime.
pub fn convert_document(
    doc: &PirDocument,
    source: &RuntimeKind,
    target: &RuntimeKind,
    policy: &ConversionPolicy,
) -> parallax_core::Result<PirDocument> {
    let mut out = PirDocument::new();
    out.schema = doc.schema;
    out.metadata = doc.metadata.clone();
    out.metadata.insert(
        "migrated_from".into(),
        serde_json::json!(source.to_string()),
    );
    out.metadata
        .insert("migrated_to".into(), serde_json::json!(target.to_string()));
    for (k, v) in &doc.bindings {
        out.set_binding(k.clone(), convert_value(v, source, target, policy)?);
    }
    // Object graph: convert shallowly as well.
    for (id, v) in &doc.objects {
        out.objects
            .insert(*id, convert_value(v, source, target, policy)?);
    }
    out.roots = doc.roots.clone();
    Ok(out)
}

/// Convert a single value.
pub fn convert_value(
    value: &PirValue,
    source: &RuntimeKind,
    target: &RuntimeKind,
    policy: &ConversionPolicy,
) -> parallax_core::Result<PirValue> {
    match value {
        PirValue::Int { v } => convert_int(v, target, policy),
        PirValue::Tuple { v: xs } => {
            let converted = map_vec(xs, source, target, policy)?;
            if matches!(target, RuntimeKind::JavaScript) {
                Ok(PirValue::List { v: converted })
            } else {
                Ok(PirValue::Tuple { v: converted })
            }
        }
        PirValue::Set { v: xs } => {
            let converted = map_vec(xs, source, target, policy)?;
            if matches!(target, RuntimeKind::JavaScript) {
                Ok(PirValue::List { v: converted })
            } else {
                Ok(PirValue::Set { v: converted })
            }
        }
        PirValue::List { v: xs } => Ok(PirValue::List {
            v: map_vec(xs, source, target, policy)?,
        }),
        PirValue::Map { entries } => {
            let mut out = Vec::with_capacity(entries.len());
            for e in entries {
                out.push(PirMapEntry {
                    key: convert_value(&e.key, source, target, policy)?,
                    value: convert_value(&e.value, source, target, policy)?,
                });
            }
            Ok(PirValue::Map { entries: out })
        }
        PirValue::BigInt { v } => {
            if matches!(target, RuntimeKind::Python) {
                Ok(PirValue::Int {
                    v: PirInteger::from_decimal(v.clone()),
                })
            } else {
                Ok(PirValue::BigInt { v: v.clone() })
            }
        }
        PirValue::Function { name, descriptor } => {
            if policy.reject_unsupported {
                Err(ParallaxError::new(
                    ErrorCode::UnsupportedValue,
                    format!("cannot migrate function {:?}", name),
                )
                .with_source("parallax-migrate")
                .with_operation("convert_value"))
            } else {
                Ok(PirValue::Unsupported {
                    reason: "function migration unsupported".into(),
                    repr: descriptor.clone(),
                    type_name: Some("function".into()),
                })
            }
        }
        PirValue::Unsupported {
            reason,
            repr,
            type_name,
        } => {
            if policy.reject_unsupported {
                Err(
                    ParallaxError::new(ErrorCode::UnsupportedValue, reason.clone())
                        .with_source("parallax-migrate")
                        .with_operation("convert_value"),
                )
            } else {
                Ok(PirValue::Unsupported {
                    reason: reason.clone(),
                    repr: repr.clone(),
                    type_name: type_name.clone(),
                })
            }
        }
        other => Ok(other.clone()),
    }
}

fn map_vec(
    xs: &[PirValue],
    source: &RuntimeKind,
    target: &RuntimeKind,
    policy: &ConversionPolicy,
) -> parallax_core::Result<Vec<PirValue>> {
    xs.iter()
        .map(|x| convert_value(x, source, target, policy))
        .collect()
}

fn convert_int(
    v: &PirInteger,
    target: &RuntimeKind,
    policy: &ConversionPolicy,
) -> parallax_core::Result<PirValue> {
    if !matches!(target, RuntimeKind::JavaScript) {
        return Ok(PirValue::Int { v: v.clone() });
    }
    let loss = v.js_number_loss();
    match loss {
        SemanticLoss::None => Ok(PirValue::Int { v: v.clone() }),
        SemanticLoss::Lossy => {
            if policy.prefer_bigint {
                Ok(PirValue::BigInt {
                    v: v.decimal.clone(),
                })
            } else if policy.allow_lossy {
                match v.as_i128() {
                    Some(i) => Ok(PirValue::Float { v: i as f64 }),
                    // Beyond i128: do not emit NaN (which used to collapse to JSON null).
                    None => Ok(PirValue::BigInt {
                        v: v.decimal.clone(),
                    }),
                }
            } else {
                Err(ParallaxError::new(
                    ErrorCode::MigrationRejected,
                    format!("integer {} loses precision as JS Number", v.decimal),
                )
                .with_source("parallax-migrate")
                .with_operation("convert_int"))
            }
        }
        other => Err(ParallaxError::new(
            ErrorCode::MigrationRejected,
            format!("unexpected integer loss class {other}"),
        )
        .with_source("parallax-migrate")
        .with_operation("convert_int")),
    }
}
