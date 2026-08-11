//! Cross-runtime migration engine and semantic-loss analysis.

#![deny(unsafe_code)]
#![warn(missing_docs)]

mod analyze;
mod contract;
mod convert;
mod report;

pub use analyze::{analyze_document, analyze_value, LossFinding};
pub use contract::{
    analyze_contract, analyze_ues_contract, require_contract, ContractAnalysis, ContractFinding,
    MigrationContract, RequiredSemantic,
};
pub use convert::{convert_document, convert_value};
pub use report::{MigrationReport, MigrationTimings};

use parallax_core::{
    ConversionPolicy, ErrorCode, MigrationId, ParallaxError, Remediation, RuntimeKind, SemanticLoss,
};
use parallax_ir::PirDocument;
use std::time::Instant;

/// Perform an offline PIR migration analysis + conversion.
pub fn migrate_document(
    source_runtime: RuntimeKind,
    target_runtime: RuntimeKind,
    doc: &PirDocument,
    policy: &ConversionPolicy,
) -> parallax_core::Result<(PirDocument, MigrationReport)> {
    let id = MigrationId::new();
    let t0 = Instant::now();
    let findings = analyze_document(doc, &source_runtime, &target_runtime, policy);
    let analyze_us = t0.elapsed().as_micros() as u64;

    let worst = findings
        .iter()
        .map(|f| f.loss)
        .fold(SemanticLoss::None, SemanticLoss::worsen);

    if worst.blocks_default_migration() && !policy.allows(worst) {
        let detail = findings
            .iter()
            .filter(|f| f.loss.blocks_default_migration())
            .map(|f| format!("{} @ {}: {}", f.loss, f.path, f.message))
            .collect::<Vec<_>>()
            .join("\n");
        return Err(ParallaxError::new(
            ErrorCode::MigrationRejected,
            format!(
                "migration {} → {} rejected due to semantic loss ({worst})",
                source_runtime, target_runtime
            ),
        )
        .with_source("parallax-migrate")
        .with_operation("migrate_document")
        .context("findings", detail)
        .remediate(Remediation::with_detail(
            "Re-run with --allow-lossy if the loss is acceptable",
            "Or adjust values / use BigInt-friendly targets",
        )));
    }

    // Unsupported values: either reject or keep as Unsupported nodes.
    if policy.reject_unsupported && findings.iter().any(|f| f.loss == SemanticLoss::Unsupported) {
        return Err(ParallaxError::new(
            ErrorCode::UnsupportedValue,
            "migration contains unsupported values",
        )
        .with_source("parallax-migrate")
        .with_operation("migrate_document"));
    }

    let t1 = Instant::now();
    let converted = convert_document(doc, &source_runtime, &target_runtime, policy)?;
    let convert_us = t1.elapsed().as_micros() as u64;

    let report = MigrationReport {
        id,
        source_runtime,
        target_runtime,
        policy: policy.clone(),
        findings,
        worst_loss: worst,
        timings: MigrationTimings {
            analyze_us,
            convert_us,
            capture_us: None,
            restore_us: None,
            total_us: analyze_us + convert_us,
        },
        success: true,
        notes: Vec::new(),
    };
    Ok((converted, report))
}

#[cfg(test)]
mod tests {
    use super::*;
    use parallax_ir::{PirInteger, PirValue};

    #[test]
    fn rejects_unsafe_int_without_bigint_or_lossy() {
        let mut doc = PirDocument::new();
        doc.set_binding(
            "n",
            PirValue::Int {
                v: PirInteger::from_decimal("9007199254740993"),
            },
        );
        let policy = ConversionPolicy {
            prefer_bigint: false,
            allow_lossy: false,
            ..ConversionPolicy::default()
        };
        let err = migrate_document(RuntimeKind::Python, RuntimeKind::JavaScript, &doc, &policy)
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::MigrationRejected);
    }

    #[test]
    fn prefer_bigint_converts_unsafe_int() {
        let mut doc = PirDocument::new();
        doc.set_binding(
            "n",
            PirValue::Int {
                v: PirInteger::from_decimal("9007199254740993"),
            },
        );
        let (out, report) = migrate_document(
            RuntimeKind::Python,
            RuntimeKind::JavaScript,
            &doc,
            &ConversionPolicy::default(),
        )
        .unwrap();
        assert!(report.success);
        assert!(matches!(out.binding("n"), Some(PirValue::BigInt { .. })));
    }

    #[test]
    fn reject_unsupported_policy_fails_on_function() {
        let mut doc = PirDocument::new();
        doc.set_binding(
            "fn",
            PirValue::Function {
                name: Some("f".into()),
                descriptor: "<fn>".into(),
            },
        );
        let err = migrate_document(
            RuntimeKind::Python,
            RuntimeKind::JavaScript,
            &doc,
            &ConversionPolicy::strict(),
        )
        .unwrap_err();
        assert!(matches!(
            err.code,
            ErrorCode::MigrationRejected | ErrorCode::UnsupportedValue
        ));
    }

    #[test]
    fn keeps_unsupported_under_default_policy() {
        let mut doc = PirDocument::new();
        doc.set_binding(
            "fn",
            PirValue::Function {
                name: Some("f".into()),
                descriptor: "<fn>".into(),
            },
        );
        let (out, report) = migrate_document(
            RuntimeKind::Python,
            RuntimeKind::JavaScript,
            &doc,
            &ConversionPolicy::default(),
        )
        .unwrap();
        assert!(report.success);
        assert!(matches!(
            out.binding("fn"),
            Some(PirValue::Unsupported { .. })
        ));
    }

    #[test]
    fn migrates_demo_state() {
        use indexmap::IndexMap;
        use parallax_ir::PirMapEntry;
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
        let (out, report) = migrate_document(
            RuntimeKind::Python,
            RuntimeKind::JavaScript,
            &doc,
            &ConversionPolicy::default(),
        )
        .unwrap();
        assert!(report.success);
        assert!(out.binding("state").is_some());
    }
}
