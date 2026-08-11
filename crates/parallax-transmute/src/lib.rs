//! Parallax Transmute — semantic project migration engine.
//!
//! Pipeline: analyze → ProjectGraph + PUIR → plan → codegen → build → test → repair → report.
//! Do not treat migration as file-by-file text translation.

#![deny(unsafe_code)]
#![allow(missing_docs)]

mod analyze;
mod codegen;
mod deps;
mod frontend;
mod infer;
mod options;
mod origin;
mod packs;
mod plan;
mod repair;
mod report;
mod score;
mod workspace;

pub use analyze::analyze_project;
pub use deps::{ChosenDependency, DepEquivalent, DepMapping, DependencyMapDb};
pub use options::{TargetStyle, TransmuteOptions};
pub use origin::{lookup_origin, SourceMapEntry, SourceMapFile};
pub use plan::{MigrationPlan, MigrationPlanner};
pub use report::{BehavioralComparison, CompatibilityScores, TransmuteReport};
pub use workspace::TransmuteWorkspace;

use chrono::Utc;
use codegen::rust::generate_rust_project;
use frontend::is_project_root;
use options::TransmuteOptions as Opts;
use packs::select_pack;
use parallax_core::{ErrorCode, ParallaxError, Remediation};
use parallax_project::{SourceLanguage, TargetLanguage};
use plan::MigrationPlanner as Planner;
use repair::repair_loop;
use score::compute_compatibility;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use tracing::{info, warn};

/// Outcome of a Transmute migration.
#[derive(Debug)]
pub struct TransmuteResult {
    /// Output directory (even on dry-run: planned path).
    pub output_dir: PathBuf,
    /// Full report.
    pub report: TransmuteReport,
    /// Whether quality gates passed.
    pub gates_passed: bool,
}

/// Detect whether `path` should use Transmute (project) vs value PIR migrate.
pub fn looks_like_project(path: &Path) -> bool {
    is_project_root(path)
}

/// Run the full Transmute pipeline.
pub async fn transmute_project(opts: &Opts) -> Result<TransmuteResult, ParallaxError> {
    let t0 = Instant::now();
    let root = opts.source.canonicalize().map_err(|e| {
        ParallaxError::new(
            ErrorCode::InvalidArgument,
            format!("invalid source path: {e}"),
        )
        .with_source("parallax-transmute")
        .with_operation("transmute_project")
    })?;

    info!(root = %root.display(), "transmute.analyze.begin");
    let analysis = analyze_project(&root, opts.from.clone()).await?;
    analysis.validate()?;

    let target = opts.to.clone();
    if !matches!(
        (&analysis.primary_language, &target),
        (
            SourceLanguage::TypeScript | SourceLanguage::JavaScript,
            TargetLanguage::Rust
        ) | (SourceLanguage::Python, TargetLanguage::Rust)
    ) {
        // Allow planning/dry-run for unsupported pairs with clear Unsupported in report,
        // but only TypeScript/JS → Rust is fully implemented for generation.
        if !matches!(
            (&analysis.primary_language, &target),
            (
                SourceLanguage::TypeScript | SourceLanguage::JavaScript,
                TargetLanguage::Rust
            )
        ) {
            if matches!(target, TargetLanguage::Rust)
                && matches!(analysis.primary_language, SourceLanguage::Python)
            {
                // Python → Rust scaffold path (partial).
            } else if !opts.dry_run {
                return Err(ParallaxError::new(
                    ErrorCode::UnsupportedValue,
                    format!(
                        "project migration {} → {} is not implemented yet (supported: typescript|javascript → rust)",
                        analysis.primary_language, target
                    ),
                )
                .with_source("parallax-transmute")
                .with_operation("transmute_project")
                .remediate(Remediation::new(
                    "Use --to rust with a TypeScript/JavaScript project, or --dry-run to inspect analysis",
                )));
            }
        }
    }

    let dep_db = DependencyMapDb::builtin();
    let pack = select_pack(&analysis, &target);
    let planner = Planner::new(dep_db, pack);
    let plan = planner.plan(&analysis, &target, opts)?;

    let scores = compute_compatibility(&analysis, &plan);
    let output_dir = opts.output.clone().unwrap_or_else(|| {
        let name = root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("project");
        root.parent()
            .unwrap_or(Path::new("."))
            .join(format!("{name}-{}", target.as_str()))
    });

    let mut report = TransmuteReport::new(&analysis, &plan, &scores);
    report.timings_ms.analyze = t0.elapsed().as_millis() as u64;

    if opts.dry_run {
        report.dry_run = true;
        report.output_dir = Some(output_dir.display().to_string());
        report.finalize_quality();
        return Ok(TransmuteResult {
            output_dir,
            report,
            gates_passed: true,
        });
    }

    // Workspace
    let ws = TransmuteWorkspace::create(&root, &analysis, &plan)?;
    ws.write_plan(&plan)?;

    let t_gen = Instant::now();
    let gen = match target {
        TargetLanguage::Rust => generate_rust_project(&analysis, &plan, &output_dir, opts)?,
        other => {
            return Err(ParallaxError::new(
                ErrorCode::UnsupportedValue,
                format!("codegen for target {other} is not implemented"),
            )
            .with_source("parallax-transmute")
            .with_operation("codegen"));
        }
    };
    report.timings_ms.generate = t_gen.elapsed().as_millis() as u64;
    report.translated_files = gen.translated_files.clone();
    report.generated_files = gen.generated_files.clone();
    report.manual_reviews = gen.manual_reviews.clone();
    report.source_maps = gen.source_maps.clone();
    report.unsupported_regions = gen.unsupported_regions.clone();

    // Write source maps
    let map_path = output_dir.join(".plxmap.json");
    fs::write(
        &map_path,
        serde_json::to_string_pretty(&gen.source_maps).map_err(|e| {
            ParallaxError::new(ErrorCode::SerializationFailure, e.to_string())
                .with_source("parallax-transmute")
        })?,
    )?;

    // Build / repair / test for Rust
    if matches!(target, TargetLanguage::Rust)
        && (opts.verify || opts.require_build || opts.require_tests)
    {
        let t_build = Instant::now();
        let repair = repair_loop(&output_dir, opts.max_repair_passes)?;
        report.repair_passes = repair.passes;
        report.build_success = Some(repair.build_ok);
        report.timings_ms.build = t_build.elapsed().as_millis() as u64;

        if opts.verify || opts.require_tests {
            let t_test = Instant::now();
            let test = run_cargo_test(&output_dir)?;
            report.test_results = Some(test.clone());
            report.timings_ms.test = t_test.elapsed().as_millis() as u64;

            if opts.verify {
                let mut behavioral = compare_behavior(&root, &output_dir, &analysis)?;
                behavioral.migrated_tests_passed = test.passed;
                behavioral.migrated_tests_total = test.passed + test.failed;
                report.behavioral = Some(behavioral);
            }
        }
    }

    report.output_dir = Some(output_dir.display().to_string());
    report.finalize_quality();

    // Write reports
    let json_report = output_dir.join("parallax-report.json");
    fs::write(
        &json_report,
        serde_json::to_string_pretty(&report)
            .map_err(|e| ParallaxError::new(ErrorCode::SerializationFailure, e.to_string()))?,
    )?;
    let md_report = output_dir.join("PARALLAX_MIGRATION.md");
    fs::write(&md_report, report.to_markdown())?;

    ws.write_diagnostics(&report)?;

    let gates_passed = evaluate_gates(opts, &report);
    if !gates_passed {
        warn!("transmute quality gates failed");
    }

    report.timings_ms.total = t0.elapsed().as_millis() as u64;
    // rewrite report with total
    fs::write(
        &json_report,
        serde_json::to_string_pretty(&report).unwrap_or_default(),
    )?;
    fs::write(&md_report, report.to_markdown())?;

    Ok(TransmuteResult {
        output_dir,
        report,
        gates_passed,
    })
}

fn evaluate_gates(opts: &Opts, report: &TransmuteReport) -> bool {
    if opts.require_build && !report.build_success.unwrap_or(false) {
        return false;
    }
    if opts.require_tests {
        if let Some(t) = &report.test_results {
            if t.failed > 0 || t.passed == 0 {
                return false;
            }
        } else {
            return false;
        }
    }
    if let Some(min) = opts.min_confidence {
        if report.overall_confidence_score < min {
            return false;
        }
    }
    if opts.fail_on_unsupported && !report.unsupported_regions.is_empty() {
        return false;
    }
    true
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TestResults {
    /// Passed.
    pub passed: u32,
    /// Failed.
    pub failed: u32,
    /// Raw output tail.
    pub output_tail: String,
}

fn run_cargo_test(dir: &Path) -> Result<TestResults, ParallaxError> {
    let out = Command::new("cargo")
        .args(["test", "--", "--nocapture"])
        .current_dir(dir)
        .output()
        .map_err(|e| {
            ParallaxError::new(
                ErrorCode::ExecutionFailure,
                format!("cargo test failed to start: {e}"),
            )
            .with_source("parallax-transmute")
            .with_operation("run_cargo_test")
        })?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}\n{stderr}");
    let mut passed = 0u32;
    let mut failed = 0u32;
    // cargo test summary: "test result: ok. N passed; M failed"
    if let Some(cap) = regex_test_summary(&combined) {
        passed = cap.0;
        failed = cap.1;
    } else if out.status.success() {
        passed = 1;
    } else {
        failed = 1;
    }
    let tail: String = combined
        .chars()
        .rev()
        .take(2000)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    Ok(TestResults {
        passed,
        failed,
        output_tail: tail,
    })
}

fn regex_test_summary(s: &str) -> Option<(u32, u32)> {
    let re = regex::Regex::new(r"(\d+) passed;\s*(\d+) failed").ok()?;
    let caps = re.captures(s)?;
    Some((caps[1].parse().ok()?, caps[2].parse().ok()?))
}

fn compare_behavior(
    source_root: &Path,
    _target_root: &Path,
    analysis: &parallax_project::ProjectAnalysis,
) -> Result<BehavioralComparison, ParallaxError> {
    // Run source tests if npm test exists; compare pass counts. Honest: may be partial.
    let mut original_passed = 0u32;
    let mut original_total = 0u32;
    let pkg = source_root.join("package.json");
    if pkg.exists() {
        let npm = Command::new("npm")
            .args(["test", "--silent"])
            .current_dir(source_root)
            .output();
        if let Ok(out) = npm {
            let text = format!(
                "{}\n{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            // node:test / vitest style rough parse
            if let Some((p, f)) = regex_test_summary(&text) {
                original_passed = p;
                original_total = p + f;
            } else if out.status.success() {
                original_passed = analysis.graph.test_file_count() as u32;
                original_total = original_passed;
            }
        }
    }
    Ok(BehavioralComparison {
        original_tests_passed: original_passed,
        original_tests_total: original_total,
        migrated_tests_passed: 0, // filled by caller if needed
        migrated_tests_total: 0,
        differences: Vec::new(),
        compared_at: Utc::now(),
    })
}
