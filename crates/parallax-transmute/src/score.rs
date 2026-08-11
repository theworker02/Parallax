//! Compatibility scores derived from real analysis (never invented).

use crate::plan::MigrationPlan;
use crate::report::CompatibilityScores;
use parallax_project::ProjectAnalysis;
use parallax_puir::{Confidence, PuirItem};

/// Compute scores from graph / PUIR / plan evidence.
pub fn compute_compatibility(analysis: &ProjectAnalysis, plan: &MigrationPlan) -> CompatibilityScores {
    let lang = score_language_semantics(analysis);
    let deps = score_dependencies(plan);
    let fw = score_framework(plan);
    let tests = score_tests(analysis);
    let config = score_config(analysis, plan);

    CompatibilityScores {
        language_semantics: lang,
        dependencies: deps,
        framework_mappings: fw,
        tests,
        configuration: config,
    }
}

fn score_language_semantics(analysis: &ProjectAnalysis) -> f64 {
    let mut total = 0u32;
    let mut supported = 0u32;
    for module in analysis.puir.modules.values() {
        for item in &module.items {
            total += 1;
            match item {
                PuirItem::Unsupported { .. } => {}
                _ => supported += 1,
            }
        }
    }
    if total == 0 {
        return 0.0;
    }
    (supported as f64 / total as f64) * 100.0
}

fn score_dependencies(plan: &MigrationPlan) -> f64 {
    if plan.dependencies.is_empty() {
        return 100.0;
    }
    let sum: f64 = plan.dependencies.iter().map(|d| d.confidence).sum();
    (sum / plan.dependencies.len() as f64) * 100.0
}

fn score_framework(plan: &MigrationPlan) -> f64 {
    if plan.framework_mappings.is_empty() {
        return 100.0;
    }
    // Pack present ⇒ high baseline; reduce if many unresolved deps.
    let unresolved = plan
        .dependencies
        .iter()
        .filter(|d| d.target.is_none())
        .count();
    let base = 90.0;
    let penalty = (unresolved as f64) * 5.0;
    (base - penalty).clamp(0.0, 100.0)
}

fn score_tests(analysis: &ProjectAnalysis) -> f64 {
    let tests = analysis.graph.test_file_count();
    if tests == 0 {
        return 100.0; // nothing to migrate
    }
    // Presence of test files we will attempt to translate.
    let translated_tests = analysis
        .puir
        .modules
        .values()
        .filter(|m| m.path.contains("test") || m.path.contains("spec"))
        .count();
    if translated_tests == 0 {
        return 50.0;
    }
    ((translated_tests as f64 / tests as f64) * 100.0).clamp(0.0, 100.0)
}

fn score_config(analysis: &ProjectAnalysis, plan: &MigrationPlan) -> f64 {
    let has_pkg = analysis
        .graph
        .files
        .iter()
        .any(|f| f.path.ends_with("package.json") || f.path.ends_with("pyproject.toml"));
    if !has_pkg {
        return 80.0;
    }
    if plan.infrastructure.iter().any(|f| f.contains("Cargo.toml") || f.contains("go.mod")) {
        90.0
    } else {
        60.0
    }
}

/// Aggregate confidence label from scores + unsupported count.
pub fn overall_label(scores: &CompatibilityScores, unsupported: usize, reviews: usize) -> Confidence {
    let avg = (scores.language_semantics
        + scores.dependencies
        + scores.framework_mappings
        + scores.tests
        + scores.configuration)
        / 5.0;
    if unsupported > 0 && avg < 70.0 {
        return Confidence::Low;
    }
    if unsupported > 3 || reviews > 10 {
        return Confidence::Medium;
    }
    if avg >= 95.0 && unsupported == 0 {
        Confidence::Exact
    } else if avg >= 85.0 {
        Confidence::High
    } else if avg >= 70.0 {
        Confidence::Medium
    } else {
        Confidence::Low
    }
}

/// Estimate band for CLI.
pub fn estimate_band(scores: &CompatibilityScores) -> &'static str {
    let avg = (scores.language_semantics
        + scores.dependencies
        + scores.framework_mappings
        + scores.tests
        + scores.configuration)
        / 5.0;
    if avg >= 85.0 {
        "HIGH CONFIDENCE"
    } else if avg >= 70.0 {
        "MEDIUM CONFIDENCE"
    } else {
        "LOW CONFIDENCE — expect manual review"
    }
}
