//! Transmute CLI / API options.

use parallax_project::{SourceLanguage, TargetLanguage};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Target code style.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetStyle {
    /// Preserve source shapes closely.
    Literal,
    /// Prefer idiomatic target conventions (default).
    Idiomatic,
}

impl TargetStyle {
    /// Parse.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "literal" => Some(Self::Literal),
            "idiomatic" => Some(Self::Idiomatic),
            _ => None,
        }
    }
}

/// Options for a project migration.
#[derive(Clone, Debug)]
pub struct TransmuteOptions {
    /// Source path (file or directory).
    pub source: PathBuf,
    /// Optional explicit source language.
    pub from: Option<SourceLanguage>,
    /// Target language.
    pub to: TargetLanguage,
    /// Output directory.
    pub output: Option<PathBuf>,
    /// Dry run — plan only.
    pub dry_run: bool,
    /// Strict conversion policy.
    pub strict: bool,
    /// Interactive prompts (reserved; currently unused).
    pub interactive: bool,
    /// Preserve source layout when possible.
    pub preserve_layout: bool,
    /// Target style.
    pub target_style: TargetStyle,
    /// Write detailed report paths.
    pub report: bool,
    /// Build + test + behavioral verify.
    pub verify: bool,
    /// Require successful build.
    pub require_build: bool,
    /// Require tests to pass.
    pub require_tests: bool,
    /// Minimum overall confidence \[0,1\].
    pub min_confidence: Option<f64>,
    /// Fail if any unsupported region.
    pub fail_on_unsupported: bool,
    /// Languages / file kinds to keep untranslated.
    pub keep: Vec<String>,
    /// Max compile-repair passes.
    pub max_repair_passes: u32,
    /// Incremental update mode.
    pub update: bool,
}

impl Default for TransmuteOptions {
    fn default() -> Self {
        Self {
            source: PathBuf::from("."),
            from: None,
            to: TargetLanguage::Rust,
            output: None,
            dry_run: false,
            strict: false,
            interactive: false,
            preserve_layout: false,
            target_style: TargetStyle::Idiomatic,
            report: true,
            verify: false,
            require_build: false,
            require_tests: false,
            min_confidence: None,
            fail_on_unsupported: false,
            keep: Vec::new(),
            max_repair_passes: 3,
            update: false,
        }
    }
}
