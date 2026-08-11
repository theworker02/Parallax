//! Migration reports (JSON + Markdown).

use crate::origin::SourceMapFile;
use crate::plan::MigrationPlan;
use crate::score::{estimate_band, overall_label};
use crate::TestResults;
use chrono::{DateTime, Utc};
use parallax_project::ProjectAnalysis;
use parallax_puir::Confidence;
use serde::{Deserialize, Serialize};

/// Compatibility percentages from measured analysis.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompatibilityScores {
    /// Language semantics coverage %.
    pub language_semantics: f64,
    /// Dependency mapping %.
    pub dependencies: f64,
    /// Framework mapping %.
    pub framework_mappings: f64,
    /// Test migration readiness %.
    pub tests: f64,
    /// Configuration translation %.
    pub configuration: f64,
}

/// Behavioral comparison summary.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BehavioralComparison {
    /// Original tests passed.
    pub original_tests_passed: u32,
    /// Original tests total.
    pub original_tests_total: u32,
    /// Migrated tests passed.
    pub migrated_tests_passed: u32,
    /// Migrated tests total.
    pub migrated_tests_total: u32,
    /// Differences.
    pub differences: Vec<String>,
    /// Timestamp.
    pub compared_at: DateTime<Utc>,
}

/// Timing breakdown (milliseconds).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TransmuteTimings {
    /// Analyze.
    pub analyze: u64,
    /// Generate.
    pub generate: u64,
    /// Build/repair.
    pub build: u64,
    /// Test.
    pub test: u64,
    /// Total.
    pub total: u64,
}

/// Manual review location.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManualReview {
    /// Target file.
    pub file: String,
    /// Line if known.
    pub line: Option<u32>,
    /// Reason.
    pub reason: String,
    /// Original source if known.
    pub origin: Option<String>,
}

/// Full transmute report.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransmuteReport {
    /// Schema.
    pub version: u32,
    /// When created.
    pub created_at: DateTime<Utc>,
    /// Source root.
    pub source_root: String,
    /// Source language.
    pub source_language: String,
    /// Target language.
    pub target_language: String,
    /// Detected framework.
    pub framework: Option<String>,
    /// Database layer.
    pub database: Option<String>,
    /// File counts.
    pub files_analyzed: usize,
    /// Source files.
    pub source_files: usize,
    /// Tests.
    pub tests: usize,
    /// Dependencies.
    pub dependencies: usize,
    /// Entrypoints.
    pub entrypoints: usize,
    /// Compatibility scores.
    pub compatibility: CompatibilityScores,
    /// Estimate band.
    pub estimate: String,
    /// Plan pack id.
    pub pack_id: String,
    /// Dependency replacements.
    pub dependency_replacements: Vec<(String, String)>,
    /// Translated files.
    pub translated_files: Vec<String>,
    /// Generated infrastructure.
    pub generated_files: Vec<String>,
    /// Manual reviews.
    pub manual_reviews: Vec<ManualReview>,
    /// Unsupported regions.
    pub unsupported_regions: Vec<String>,
    /// Source maps.
    pub source_maps: SourceMapFile,
    /// Build success.
    pub build_success: Option<bool>,
    /// Test results.
    pub test_results: Option<TestResults>,
    /// Behavioral comparison.
    pub behavioral: Option<BehavioralComparison>,
    /// Repair pass log.
    pub repair_passes: Vec<String>,
    /// Dry run?
    pub dry_run: bool,
    /// Output dir.
    pub output_dir: Option<String>,
    /// Overall confidence label.
    pub overall_confidence: String,
    /// Overall score 0–1.
    pub overall_confidence_score: f64,
    /// Timings.
    pub timings_ms: TransmuteTimings,
    /// Language mix.
    pub language_mix: indexmap::IndexMap<String, f64>,
}

impl TransmuteReport {
    /// Seed report from analysis + plan + scores.
    pub fn new(analysis: &ProjectAnalysis, plan: &MigrationPlan, scores: &CompatibilityScores) -> Self {
        let replacements: Vec<(String, String)> = plan
            .dependencies
            .iter()
            .filter_map(|d| d.target.clone().map(|t| (d.source.clone(), t)))
            .collect();
        let review_count_estimate = plan.dependencies.iter().filter(|d| d.manual_review).count();
        Self {
            version: 1,
            created_at: Utc::now(),
            source_root: analysis.root.clone(),
            source_language: analysis.primary_language.to_string(),
            target_language: plan.target_language.clone(),
            framework: analysis.framework.clone(),
            database: analysis.database.clone(),
            files_analyzed: analysis.graph.files.len(),
            source_files: analysis.graph.source_file_count(),
            tests: analysis.graph.test_file_count(),
            dependencies: analysis.graph.packages.len(),
            entrypoints: analysis.graph.entrypoints.len(),
            compatibility: scores.clone(),
            estimate: estimate_band(scores).into(),
            pack_id: plan.pack_id.clone(),
            dependency_replacements: replacements,
            translated_files: Vec::new(),
            generated_files: Vec::new(),
            manual_reviews: plan
                .dependencies
                .iter()
                .filter(|d| d.manual_review)
                .map(|d| ManualReview {
                    file: format!("dependency:{}", d.source),
                    line: None,
                    reason: d.notes.clone(),
                    origin: None,
                })
                .collect(),
            unsupported_regions: Vec::new(),
            source_maps: SourceMapFile::default(),
            build_success: None,
            test_results: None,
            behavioral: None,
            repair_passes: Vec::new(),
            dry_run: false,
            output_dir: None,
            overall_confidence: Confidence::Medium.label().into(),
            overall_confidence_score: 0.0,
            timings_ms: TransmuteTimings::default(),
            language_mix: analysis.language_mix.clone(),
        }
        .tap_estimate(review_count_estimate)
    }

    fn tap_estimate(self, _reviews: usize) -> Self {
        self
    }

    /// Finalize confidence fields.
    pub fn finalize_quality(&mut self) {
        let label = overall_label(
            &self.compatibility,
            self.unsupported_regions.len(),
            self.manual_reviews.len(),
        );
        self.overall_confidence = label.label().into();
        self.overall_confidence_score = label.score();
        if let Some(b) = self.behavioral.as_mut() {
            if let Some(t) = &self.test_results {
                b.migrated_tests_passed = t.passed;
                b.migrated_tests_total = t.passed + t.failed;
            }
        }
    }

    /// Markdown report body.
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str("# PARALLAX MIGRATION REPORT\n\n");
        md.push_str(&format!("- Source: `{}` ({})\n", self.source_root, self.source_language));
        md.push_str(&format!("- Target: {}\n", self.target_language));
        if let Some(fw) = &self.framework {
            md.push_str(&format!("- Framework: {fw}\n"));
        }
        md.push_str(&format!("- Pack: {}\n", self.pack_id));
        md.push_str(&format!("- Estimate: {}\n", self.estimate));
        md.push_str(&format!("- Overall confidence: {}\n\n", self.overall_confidence));
        md.push_str("## Compatibility\n\n");
        md.push_str(&format!(
            "| Area | Score |\n|---|---|\n| Language semantics | {:.1}% |\n| Dependencies | {:.1}% |\n| Framework mappings | {:.1}% |\n| Tests | {:.1}% |\n| Configuration | {:.1}% |\n\n",
            self.compatibility.language_semantics,
            self.compatibility.dependencies,
            self.compatibility.framework_mappings,
            self.compatibility.tests,
            self.compatibility.configuration
        ));
        md.push_str("## Dependency replacements\n\n");
        for (a, b) in &self.dependency_replacements {
            md.push_str(&format!("- `{a}` → `{b}`\n"));
        }
        md.push_str("\n## Manual review\n\n");
        if self.manual_reviews.is_empty() {
            md.push_str("None.\n");
        } else {
            for r in &self.manual_reviews {
                md.push_str(&format!("- `{}`: {}\n", r.file, r.reason));
            }
        }
        md.push_str("\n## Unsupported regions\n\n");
        if self.unsupported_regions.is_empty() {
            md.push_str("None.\n");
        } else {
            for u in &self.unsupported_regions {
                md.push_str(&format!("- {u}\n"));
            }
        }
        if let Some(t) = &self.test_results {
            md.push_str(&format!(
                "\n## Tests\n\nPassed: {} / {}\n",
                t.passed,
                t.passed + t.failed
            ));
        }
        md.push_str(&format!(
            "\n## Timings\n\nAnalyze: {} ms · Generate: {} ms · Build: {} ms · Test: {} ms · Total: {} ms\n",
            self.timings_ms.analyze,
            self.timings_ms.generate,
            self.timings_ms.build,
            self.timings_ms.test,
            self.timings_ms.total
        ));
        md
    }
}
