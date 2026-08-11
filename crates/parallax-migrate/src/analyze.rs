//! Semantic-loss analysis for PIR → target runtime.

use parallax_core::{ConversionPolicy, RuntimeKind, SemanticLoss};
use parallax_ir::{PirDocument, PirValue};
use serde::{Deserialize, Serialize};

/// A single semantic-loss finding.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LossFinding {
    /// JSON-ish path (e.g. `state.score`).
    pub path: String,
    /// Loss classification.
    pub loss: SemanticLoss,
    /// Human message.
    pub message: String,
    /// Source type label.
    pub source_type: String,
    /// Suggested remediation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

/// Analyze an entire document.
pub fn analyze_document(
    doc: &PirDocument,
    source: &RuntimeKind,
    target: &RuntimeKind,
    policy: &ConversionPolicy,
) -> Vec<LossFinding> {
    let mut out = Vec::new();
    for (name, value) in &doc.bindings {
        analyze_value(value, name, source, target, policy, &mut out);
    }
    out
}

/// Analyze a value recursively.
pub fn analyze_value(
    value: &PirValue,
    path: &str,
    source: &RuntimeKind,
    target: &RuntimeKind,
    policy: &ConversionPolicy,
    out: &mut Vec<LossFinding>,
) {
    // `source` is part of the public analysis API for future source-specific rules.
    let _ = source;
    match value {
        PirValue::Int { v } => {
            if matches!(target, RuntimeKind::JavaScript) {
                let loss = v.js_number_loss();
                if loss != SemanticLoss::None {
                    let suggestion = if policy.prefer_bigint {
                        Some(
                            "Use BigInt in the target (prefer_bigint=true) or pass --allow-lossy"
                                .into(),
                        )
                    } else {
                        Some("Pass --allow-lossy to coerce to Number (precision loss)".into())
                    };
                    // When prefer_bigint is on, conversion will emit BigInt → Safe, not Lossy.
                    let effective = if policy.prefer_bigint && loss == SemanticLoss::Lossy {
                        SemanticLoss::Safe
                    } else {
                        loss
                    };
                    if effective != SemanticLoss::None {
                        out.push(LossFinding {
                            path: path.into(),
                            loss: if policy.prefer_bigint && loss == SemanticLoss::Lossy {
                                SemanticLoss::Safe
                            } else {
                                loss
                            },
                            message: format!(
                                "integer {} is outside JS safe integer range for Number",
                                v.decimal
                            ),
                            source_type: "int".into(),
                            suggestion,
                        });
                    }
                }
            }
        }
        PirValue::Tuple { v: xs } => {
            if matches!(target, RuntimeKind::JavaScript) {
                out.push(LossFinding {
                    path: path.into(),
                    loss: SemanticLoss::Safe,
                    message: "Python tuple → JavaScript Array (ordered sequence preserved)".into(),
                    source_type: "tuple".into(),
                    suggestion: None,
                });
            }
            for (i, x) in xs.iter().enumerate() {
                analyze_value(x, &format!("{path}[{i}]"), source, target, policy, out);
            }
        }
        PirValue::Set { v: xs } => {
            if matches!(target, RuntimeKind::JavaScript) {
                out.push(LossFinding {
                    path: path.into(),
                    loss: SemanticLoss::Safe,
                    message: "Set → JavaScript Array (set semantics become list)".into(),
                    source_type: "set".into(),
                    suggestion: Some("Target may use Set if adapter supports it".into()),
                });
            } else if matches!(target, RuntimeKind::Python) {
                // JS has no native set in our capture subset unless tagged; treat as list→set safe
            }
            for (i, x) in xs.iter().enumerate() {
                analyze_value(x, &format!("{path}[{i}]"), source, target, policy, out);
            }
        }
        PirValue::List { v: xs } => {
            for (i, x) in xs.iter().enumerate() {
                analyze_value(x, &format!("{path}[{i}]"), source, target, policy, out);
            }
        }
        PirValue::Map { entries } => {
            for (i, e) in entries.iter().enumerate() {
                match &e.key {
                    PirValue::String { v: k } => {
                        analyze_value(
                            &e.value,
                            &format!("{path}.{k}"),
                            source,
                            target,
                            policy,
                            out,
                        );
                    }
                    other => {
                        out.push(LossFinding {
                            path: format!("{path}@key[{i}]"),
                            loss: SemanticLoss::PotentiallyLossy,
                            message: format!(
                                "non-string map key ({}) may not round-trip cleanly",
                                other.type_label()
                            ),
                            source_type: other.type_label().into(),
                            suggestion: Some("Prefer string keys for cross-runtime maps".into()),
                        });
                        analyze_value(
                            &e.value,
                            &format!("{path}[{i}]"),
                            source,
                            target,
                            policy,
                            out,
                        );
                    }
                }
            }
        }
        PirValue::Function { name, .. } => {
            out.push(LossFinding {
                path: path.into(),
                loss: SemanticLoss::Unsupported,
                message: format!(
                    "function '{}' cannot be migrated between runtimes",
                    name.as_deref().unwrap_or("?")
                ),
                source_type: "function".into(),
                suggestion: Some("Migrate data only; rebind functions manually".into()),
            });
        }
        PirValue::Unsupported { reason, .. } => {
            out.push(LossFinding {
                path: path.into(),
                loss: SemanticLoss::Unsupported,
                message: reason.clone(),
                source_type: "unsupported".into(),
                suggestion: None,
            });
        }
        PirValue::Bytes { .. } => {
            if matches!(target, RuntimeKind::JavaScript) {
                out.push(LossFinding {
                    path: path.into(),
                    loss: SemanticLoss::Safe,
                    message: "bytes → Uint8Array".into(),
                    source_type: "bytes".into(),
                    suggestion: None,
                });
            }
        }
        PirValue::BigInt { .. } => {
            // BigInt is first-class in JS; in Python becomes int (Safe).
            if matches!(target, RuntimeKind::Python) {
                out.push(LossFinding {
                    path: path.into(),
                    loss: SemanticLoss::Safe,
                    message: "BigInt → Python int".into(),
                    source_type: "bigint".into(),
                    suggestion: None,
                });
            }
        }
        PirValue::Null
        | PirValue::Bool { .. }
        | PirValue::Float { .. }
        | PirValue::String { .. }
        | PirValue::Ref { .. } => {}
    }
}
