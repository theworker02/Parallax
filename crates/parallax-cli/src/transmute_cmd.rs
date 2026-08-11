//! CLI surface for Parallax Transmute (project migration).

use parallax_core::{ErrorCode, ParallaxError, Remediation};
use parallax_project::{SourceLanguage, TargetLanguage};
use parallax_transmute::{
    looks_like_project, lookup_origin, transmute_project, TargetStyle, TransmuteOptions,
    TransmuteResult,
};
use std::path::PathBuf;

pub fn should_use_transmute(
    path: &std::path::Path,
    mode: &str,
    force_project: bool,
    to: &str,
) -> bool {
    if force_project || mode == "project" {
        return true;
    }
    if mode == "value" || mode == "continuation" {
        return false;
    }
    // auto: project dirs, or targets that are languages (rust/go) rather than runtimes
    let target_is_language_only =
        TargetLanguage::parse(to).is_some() && RuntimeKindLite::parse(to).is_none();
    looks_like_project(path) || (path.is_dir() && target_is_language_only)
}

enum RuntimeKindLite {
    Python,
    JavaScript,
    Wasm,
}

impl RuntimeKindLite {
    fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "python" | "py" => Some(Self::Python),
            "javascript" | "js" | "node" => Some(Self::JavaScript),
            "wasm" | "webassembly" => Some(Self::Wasm),
            _ => None,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn cmd_transmute(
    json: bool,
    file: PathBuf,
    to: String,
    from: Option<String>,
    output: Option<PathBuf>,
    dry_run: bool,
    verify: bool,
    strict: bool,
    target_style: String,
    preserve_layout: bool,
    require_build: bool,
    require_tests: bool,
    min_confidence: Option<f64>,
    fail_on_unsupported: bool,
    keep: Vec<String>,
    update: bool,
) -> Result<(), ParallaxError> {
    let target = TargetLanguage::parse(&to).ok_or_else(|| {
        ParallaxError::new(
            ErrorCode::InvalidArgument,
            format!("unknown target language: {to}"),
        )
        .remediate(Remediation::new("Supported targets include: rust, go, python, typescript"))
    })?;
    let from_lang = from
        .as_ref()
        .map(|s| {
            SourceLanguage::parse(s).ok_or_else(|| {
                ParallaxError::new(ErrorCode::InvalidArgument, format!("unknown --from language: {s}"))
            })
        })
        .transpose()?;
    let style = TargetStyle::parse(&target_style).unwrap_or(TargetStyle::Idiomatic);

    let opts = TransmuteOptions {
        source: file,
        from: from_lang,
        to: target,
        output,
        dry_run,
        strict,
        interactive: false,
        preserve_layout,
        target_style: style,
        report: true,
        verify: verify || require_build || require_tests,
        require_build,
        require_tests,
        min_confidence,
        fail_on_unsupported,
        keep,
        max_repair_passes: 3,
        update,
    };

    let result = transmute_project(&opts).await?;
    print_transmute(json, &result)?;
    if !result.gates_passed {
        return Err(ParallaxError::new(
            ErrorCode::MigrationRejected,
            "transmute quality gates failed",
        )
        .with_source("parallax-cli")
        .with_operation("transmute")
        .context(
            "report",
            result
                .output_dir
                .join("PARALLAX_MIGRATION.md")
                .display()
                .to_string(),
        ));
    }
    Ok(())
}

fn print_transmute(json: bool, result: &TransmuteResult) -> Result<(), ParallaxError> {
    let r = &result.report;
    if json {
        println!("{}", serde_json::to_string_pretty(r).map_err(|e| {
            ParallaxError::new(ErrorCode::SerializationFailure, e.to_string())
        })?);
        return Ok(());
    }
    println!(" PARALLAX TRANSMUTE");
    println!("Analyzing project..............done");
    println!("Building semantic graph........done");
    println!("Inferring types................done");
    println!("Planning ecosystem mapping.....done");
    if r.dry_run {
        println!("Dry run........................done");
    } else {
        println!("Translating source.............done");
        println!("Generating target project......done");
        if r.build_success.is_some() {
            println!(
                "Building target.................{}",
                if r.build_success == Some(true) {
                    "done"
                } else {
                    "failed"
                }
            );
        }
        if r.test_results.is_some() {
            println!("Running tests..................done");
        }
        if r.behavioral.is_some() {
            println!("Verifying behavior.............done");
        }
    }
    println!();
    println!("Source:");
    println!(
        "  {}{}",
        r.source_language,
        r.framework
            .as_ref()
            .map(|f| format!(" / {f}"))
            .unwrap_or_default()
    );
    println!("Target:");
    println!("  {}", r.target_language);
    println!();
    println!("Files analyzed:       {}", r.files_analyzed);
    println!("Source files:         {}", r.source_files);
    println!("Tests:                {}", r.tests);
    println!("Dependencies:         {}", r.dependencies);
    println!("Entrypoints:          {}", r.entrypoints);
    if let Some(fw) = &r.framework {
        println!("Detected framework:   {fw}");
    }
    if let Some(db) = &r.database {
        println!("Database:             {db}");
    }
    println!();
    println!("Migration compatibility:");
    println!(
        "Language semantics.........{:.0}%",
        r.compatibility.language_semantics
    );
    println!(
        "Dependencies...............{:.0}%",
        r.compatibility.dependencies
    );
    println!(
        "Framework mappings.........{:.0}%",
        r.compatibility.framework_mappings
    );
    println!("Tests......................{:.0}%", r.compatibility.tests);
    println!(
        "Configuration..............{:.0}%",
        r.compatibility.configuration
    );
    println!();
    println!("Estimated migration:");
    println!("  {}", r.estimate);
    println!(
        "Potential manual review:");
    println!("  {} locations", r.manual_reviews.len());
    if !r.dependency_replacements.is_empty() {
        println!();
        println!("Dependency replacements:");
        for (a, b) in &r.dependency_replacements {
            println!("  {a:<16} → {b}");
        }
    }
    if r.dry_run {
        println!();
        println!("No files written.");
    } else {
        println!();
        println!("Files migrated:       {}", r.translated_files.len());
        if let Some(t) = &r.test_results {
            println!(
                "Tests passed:         {} / {}",
                t.passed,
                t.passed + t.failed
            );
        }
        println!("Manual reviews:       {}", r.manual_reviews.len());
        println!("Unsupported regions:  {}", r.unsupported_regions.len());
        println!("Migration quality:    {}", r.overall_confidence);
        if let Some(out) = &r.output_dir {
            println!();
            println!("Output:");
            println!("  {out}");
            println!("Report:");
            println!("  {out}/PARALLAX_MIGRATION.md");
        }
        println!();
        println!(
            "Completed in {:.1} ms",
            r.timings_ms.total as f64
        );
    }
    Ok(())
}

pub fn cmd_origin(json: bool, location: String, project: PathBuf) -> Result<(), ParallaxError> {
    let entry = lookup_origin(&project, &location)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&entry).unwrap());
    } else {
        println!("Translated from:");
        println!("{}", entry.original_file);
        println!("line {}", entry.original_line);
        println!("column {}", entry.original_column);
        println!("Semantic node:");
        println!("{}", entry.semantic_node);
    }
    Ok(())
}
