//! Event Horizon CLI — observe / impossible / dissolve / debt / …

use parallax_core::ParallaxError;
use parallax_horizon::{
    analyze_impossible, blame_line, cherry_pick, detach_status, dissolve_project, explain_barrier,
    measure_debt, optimize_migration, reconstruct_status, PreservationPolicy, ProjectObserver,
    SemanticPatch,
};
use std::path::PathBuf;

fn parse_policy(s: &Option<String>) -> Option<PreservationPolicy> {
    s.as_deref().map(|p| match p {
        "maximum-native" | "native" => PreservationPolicy::MaximumNative,
        "maximum-performance" | "performance" => PreservationPolicy::MaximumPerformance,
        "minimum-dependencies" | "min-deps" => PreservationPolicy::MinimumDependencies,
        "fastest-migration" | "fastest" => PreservationPolicy::FastestMigration,
        _ => PreservationPolicy::MaximumCompatibility,
    })
}

pub fn cmd_observe(json: bool, path: PathBuf) -> Result<(), ParallaxError> {
    let report = ProjectObserver.observe(&path).map_err(|e| {
        parallax_core::ParallaxError::new(parallax_core::ErrorCode::Io, e).with_operation("observe")
    })?;
    if json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
        return Ok(());
    }
    println!("Observatory ({})\n", report.root);
    println!("Languages:");
    for (lang, n) in &report.languages {
        println!("  {lang:16} {n} files");
    }
    if !report.frameworks.is_empty() {
        println!("\nFrameworks: {}", report.frameworks.join(", "));
    }
    if !report.dynamic_signals.is_empty() {
        println!("\nDynamic behavior:");
        for d in &report.dynamic_signals {
            println!(
                "  {} ×{}  e.g. {}",
                d.kind,
                d.count,
                d.samples.first().unwrap_or(&String::new())
            );
        }
    }
    if !report.concurrency.is_empty() {
        println!("\nConcurrency: {}", report.concurrency.join(", "));
    }
    if !report.effects.is_empty() {
        println!("Effects: {}", report.effects.join(", "));
    }
    if !report.protocols.is_empty() {
        println!("Protocols: {}", report.protocols.join(", "));
    }
    if !report.migration_barriers.is_empty() {
        println!("\nMigration barriers:");
        for b in &report.migration_barriers {
            println!("  · {b}");
        }
    }
    println!("\n({})", report.notes);
    Ok(())
}

pub fn cmd_impossible(
    json: bool,
    path: PathBuf,
    to: Option<String>,
    strategy: Option<String>,
) -> Result<(), ParallaxError> {
    let report = analyze_impossible(&path, to.as_deref(), parse_policy(&strategy))?;
    if json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
        return Ok(());
    }
    println!("Hard semantic barriers ({})\n", report.root);
    println!("Target: {}\n", report.target);
    for b in &report.barriers {
        println!(
            "[{}] {} @ {}",
            b.id,
            b.kind,
            if b.location.is_empty() {
                "?"
            } else {
                &b.location
            }
        );
        println!(
            "  strategy: {} ({:.0}%)",
            b.preferred_strategy.as_str(),
            b.confidence * 100.0
        );
        println!("  {}", b.notes);
    }
    println!("\nProposed strategy:");
    for p in &report.proposed {
        println!("  · {p}");
    }
    println!(
        "\nEstimated native target:     {:.0}%",
        report.estimated_native_pct
    );
    println!(
        "Expected compatibility layer: {:.0}%",
        report.expected_compatibility_pct
    );
    println!(
        "Polyglot requirement:         {:.0}%",
        report.polyglot_requirement_pct
    );
    if !report.strategy_options_sample.is_empty() {
        println!("\nStrategy search (sample):");
        for s in &report.strategy_options_sample {
            println!("  · {s}");
        }
    }
    println!("\n({})", report.notes);
    Ok(())
}

pub fn cmd_dissolve(json: bool, path: PathBuf, to: Option<String>) -> Result<(), ParallaxError> {
    let v = dissolve_project(&path, to.as_deref())?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    let _ = json;
    Ok(())
}

pub fn cmd_debt(json: bool, path: PathBuf, to: Option<String>) -> Result<(), ParallaxError> {
    let debt = measure_debt(&path, to.as_deref())?;
    if json {
        println!("{}", serde_json::to_string_pretty(&debt).unwrap());
        return Ok(());
    }
    println!("Compatibility debt\n");
    println!("Native target semantics.......{:.1}%", debt.native_pct);
    println!(
        "Generated compatibility.......{:.1}%",
        debt.compatibility_pct
    );
    println!(
        "Polyglot island...............{:.1}%",
        debt.polyglot_island_pct
    );
    println!("Manual compatibility..........{:.1}%", debt.manual_pct);
    println!("\nTarget purity: {:.1}%", debt.target_purity);
    println!("({})", debt.notes);
    Ok(())
}

pub fn cmd_detach(json: bool, path: PathBuf, to: Option<String>) -> Result<(), ParallaxError> {
    let v = detach_status(&path, to.as_deref())?;
    if json {
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
    } else {
        println!("{}", v["message"].as_str().unwrap_or(""));
        println!("ready: {}", v["ready"]);
    }
    Ok(())
}

pub fn cmd_reconstruct(json: bool) -> Result<(), ParallaxError> {
    let v = reconstruct_status();
    if json {
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
    } else {
        println!("{}", v["message"].as_str().unwrap_or(""));
    }
    Ok(())
}

pub fn cmd_optimize_migration(
    json: bool,
    path: PathBuf,
    to: Option<String>,
) -> Result<(), ParallaxError> {
    let v = optimize_migration(&path, to.as_deref())?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    let _ = json;
    Ok(())
}

pub fn cmd_explain_barrier(
    json: bool,
    path: PathBuf,
    id: u32,
    to: Option<String>,
) -> Result<(), ParallaxError> {
    let v = explain_barrier(&path, id, to.as_deref())?;
    if json {
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
        return Ok(());
    }
    println!("Barrier: {}", v["barrier"]["kind"]);
    println!("Why direct translation fails:\n  {}", v["why_direct_fails"]);
    println!("Resolution:\n  {}", v["resolution"]);
    println!("Strategy: {}", v["strategy"]);
    println!(
        "Confidence: {:.1}%",
        v["confidence"].as_f64().unwrap_or(0.0) * 100.0
    );
    Ok(())
}

pub fn cmd_blame(json: bool, location: String) -> Result<(), ParallaxError> {
    let v = blame_line(&location);
    if json {
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
    } else {
        println!("Location: {}", v["target_location"]);
        println!("{}", v["reason"].as_str().unwrap_or(""));
        if v["supported"] == false {
            println!("(scaffold — wire `.plxmap.json` + git for full semantic blame)");
        }
    }
    Ok(())
}

pub fn cmd_cherry_pick(json: bool, commit: String) -> Result<(), ParallaxError> {
    let v = cherry_pick(&commit);
    if json {
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
    } else {
        println!("{}", v["message"].as_str().unwrap_or(""));
    }
    Ok(())
}

pub fn cmd_patch_example(json: bool) -> Result<(), ParallaxError> {
    let p = SemanticPatch::example();
    if json {
        println!("{}", serde_json::to_string_pretty(&p).unwrap());
    } else {
        println!("Example .plxp semantic patch\n");
        println!("id: {}", p.id);
        println!("{}", p.description);
        println!("ops: {}", p.operations.len());
    }
    Ok(())
}
