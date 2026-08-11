//! Framework migration packs.

use crate::deps::ChosenDependency;
use indexmap::IndexMap;
use parallax_project::{ProjectAnalysis, TargetLanguage};
use serde::{Deserialize, Serialize};

/// A migration pack describing ecosystem remapping.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MigrationPack {
    /// Pack id (e.g. typescript-express→rust-axum).
    pub id: String,
    /// Source framework.
    pub source_framework: Option<String>,
    /// Target framework.
    pub target_framework: Option<String>,
    /// Framework mapping lines.
    pub framework_mappings: IndexMap<String, String>,
    /// Suggested project layout roots.
    pub structure_notes: Vec<String>,
    /// Testing mapping.
    pub test_mapping: String,
    /// Config mapping notes.
    pub config_notes: Vec<String>,
}

/// Select best pack for analysis → target.
pub fn select_pack(analysis: &ProjectAnalysis, target: &TargetLanguage) -> MigrationPack {
    let fw = analysis.framework.as_deref();
    match (fw, target) {
        (Some("express"), TargetLanguage::Rust) | (None, TargetLanguage::Rust)
            if matches!(
                analysis.primary_language,
                parallax_project::SourceLanguage::TypeScript
                    | parallax_project::SourceLanguage::JavaScript
            ) =>
        {
            typescript_express_to_rust_axum()
        }
        (Some("fastapi"), TargetLanguage::Rust) => python_fastapi_to_rust_axum(),
        _ => generic_pack(analysis, target),
    }
}

fn typescript_express_to_rust_axum() -> MigrationPack {
    let mut fm = IndexMap::new();
    fm.insert("express".into(), "axum".into());
    fm.insert("axios".into(), "reqwest".into());
    fm.insert("zod".into(), "serde + validator".into());
    fm.insert("dotenv".into(), "dotenvy".into());
    fm.insert("vitest".into(), "cargo test".into());
    fm.insert("jest".into(), "cargo test".into());
    MigrationPack {
        id: "typescript-express→rust-axum".into(),
        source_framework: Some("express".into()),
        target_framework: Some("axum".into()),
        framework_mappings: fm,
        structure_notes: vec![
            "src/main.rs entrypoint".into(),
            "modules as src/*.rs".into(),
            "tests/ for integration tests".into(),
        ],
        test_mapping: "vitest/jest → #[cfg(test)] / cargo test".into(),
        config_notes: vec![
            "package.json → Cargo.toml".into(),
            ".env → .env.example (names only)".into(),
        ],
    }
}

fn python_fastapi_to_rust_axum() -> MigrationPack {
    let mut fm = IndexMap::new();
    fm.insert("fastapi".into(), "axum".into());
    fm.insert("requests".into(), "reqwest".into());
    MigrationPack {
        id: "python-fastapi→rust-axum".into(),
        source_framework: Some("fastapi".into()),
        target_framework: Some("axum".into()),
        framework_mappings: fm,
        structure_notes: vec!["src/main.rs".into()],
        test_mapping: "pytest → cargo test".into(),
        config_notes: vec!["requirements.txt → Cargo.toml".into()],
    }
}

fn generic_pack(analysis: &ProjectAnalysis, target: &TargetLanguage) -> MigrationPack {
    MigrationPack {
        id: format!("{}→{}", analysis.primary_language, target),
        source_framework: analysis.framework.clone(),
        target_framework: None,
        framework_mappings: IndexMap::new(),
        structure_notes: vec!["Use target-language defaults".into()],
        test_mapping: "best-effort".into(),
        config_notes: Vec::new(),
    }
}

/// Merge pack framework mappings into chosen deps display.
pub fn pack_dep_overrides(pack: &MigrationPack) -> Vec<ChosenDependency> {
    pack.framework_mappings
        .iter()
        .map(|(src, tgt)| ChosenDependency {
            source: src.clone(),
            target: Some(tgt.clone()),
            confidence: 0.9,
            alternatives: Vec::new(),
            notes: format!("from pack {}", pack.id),
            manual_review: false,
        })
        .collect()
}
