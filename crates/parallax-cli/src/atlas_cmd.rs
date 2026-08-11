//! Atlas CLI — `plx adapters`, `analyze`, `stacks`, `mappings`, `compatibility`, …

use parallax_adapter_sdk::AdapterKind;
use parallax_atlas::{analyze_stack, builtin_registry, pair_compatibility, AdapterLockfile};
use parallax_core::{ErrorCode, ParallaxError, Remediation};
use parallax_transmute::DependencyMapDb;
use std::path::PathBuf;

pub fn cmd_adapters(json: bool, sub: Option<AdaptersSub>) -> Result<(), ParallaxError> {
    let reg = builtin_registry();
    match sub.unwrap_or(AdaptersSub::List { query: None }) {
        AdaptersSub::List { query } => {
            let mut list = reg.list();
            if let Some(q) = query {
                let q = q.to_ascii_lowercase();
                list.retain(|m| {
                    m.id.as_str().contains(&q)
                        || m.name.to_ascii_lowercase().contains(&q)
                        || m.languages
                            .iter()
                            .any(|l| l.to_ascii_lowercase().contains(&q))
                        || m.adapter_type.as_str().contains(&q)
                });
            }
            if json {
                println!("{}", serde_json::to_string_pretty(&list).unwrap());
                return Ok(());
            }
            println!("Parallax Atlas adapters ({})\n", list.len());
            print_grouped(&list);
            Ok(())
        }
        AdaptersSub::Info { id } => {
            let entry = reg.get(&id).ok_or_else(|| {
                ParallaxError::new(ErrorCode::InvalidArgument, format!("unknown adapter: {id}"))
                    .with_operation("adapters info")
                    .remediate(Remediation::new("Run `plx adapters` to list adapters"))
            })?;
            let m = entry.adapter.manifest();
            let caps = entry.adapter.capabilities();
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "manifest": m,
                        "capabilities": caps,
                    }))
                    .unwrap()
                );
                return Ok(());
            }
            println!("{}  v{}", m.name, m.version);
            println!("id:          {}", m.id);
            println!("type:        {}", m.adapter_type.as_str());
            println!("maturity:    {}", m.maturity.as_str());
            println!("conformance: {}", m.conformance.as_str());
            println!("languages:   {}", m.languages.join(", "));
            if !m.ecosystems.is_empty() {
                println!("ecosystems:  {}", m.ecosystems.join(", "));
            }
            if !m.owns.is_empty() {
                println!("owns:        {}", m.owns.join(", "));
            }
            if !m.notes.is_empty() {
                println!("notes:       {}", m.notes);
            }
            if !caps.flags.is_empty() {
                println!("\nCapabilities:");
                for f in &caps.flags {
                    println!("  {:24} {}", f.name, f.support.as_str());
                }
            }
            Ok(())
        }
        AdaptersSub::Capabilities { id } => {
            let entry = reg.get(&id).ok_or_else(|| {
                ParallaxError::new(ErrorCode::InvalidArgument, format!("unknown adapter: {id}"))
                    .with_operation("adapters capabilities")
            })?;
            let caps = entry.adapter.capabilities();
            if json {
                println!("{}", serde_json::to_string_pretty(&caps).unwrap());
                return Ok(());
            }
            let m = entry.adapter.manifest();
            println!("{} capabilities\n", m.name);
            for f in &caps.flags {
                let dots = ".".repeat((24usize).saturating_sub(f.name.len()).max(2));
                println!("{}{}{}", f.name, dots, f.support.as_str());
            }
            for c in &caps.constructs {
                let dots = ".".repeat((24usize).saturating_sub(c.construct.len()).max(2));
                println!("{}{}{}", c.construct, dots, c.support.as_str());
            }
            if caps.flags.is_empty() && caps.constructs.is_empty() {
                println!("(no capability flags declared — treat as UNKNOWN)");
            }
            Ok(())
        }
        AdaptersSub::Health => {
            let scores = reg.health_scores();
            if json {
                println!("{}", serde_json::to_string_pretty(&scores).unwrap());
                return Ok(());
            }
            println!("Adapter health scores (maturity / conformance heuristic)\n");
            let mut scores = scores;
            scores.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            for (id, score) in scores {
                println!("{id:40} {score}");
            }
            Ok(())
        }
        AdaptersSub::Update { check } => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": true,
                        "check_only": check,
                        "message": "Built-in adapters ship with Parallax; package-based updates are not implemented yet."
                    })
                );
            } else if check {
                println!("Adapter update check: built-ins are versioned with Parallax.");
                println!("Package-based adapter distribution is not implemented yet.");
            } else {
                println!("No external adapter packages to update.");
                println!("Built-in Atlas adapters update with Parallax itself.");
            }
            Ok(())
        }
        AdaptersSub::Report => {
            let scores = reg.health_scores();
            let report = serde_json::json!({
                "adapter_count": reg.len(),
                "health": scores,
                "telemetry": "local-only; no network transmission",
                "note": "Fixture pass rates / regressions will populate this report as conformance suites grow."
            });
            if json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                println!("Adapter report");
                println!("  registered: {}", reg.len());
                println!("  telemetry:  local-only (not transmitted)");
                println!("  Run `plx adapters health` for scores.");
            }
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub enum AdaptersSub {
    List { query: Option<String> },
    Info { id: String },
    Capabilities { id: String },
    Health,
    Update { check: bool },
    Report,
}

pub fn cmd_adapter_tool(json: bool, action: AdapterToolAction) -> Result<(), ParallaxError> {
    match action {
        AdapterToolAction::New { name } => {
            let msg = format!(
                "Scaffold for adapter `{name}` is not generated yet. See examples/custom-adapter/ and docs/adapters/sdk.md."
            );
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": false,
                        "unsupported": true,
                        "message": msg,
                    })
                );
            } else {
                println!("UNSUPPORTED: {msg}");
            }
            Ok(())
        }
        AdapterToolAction::Validate { path } => {
            let msg = format!(
                "Adapter validation for {} is stubbed — checks for manifest/schema/permissions are planned.",
                path.display()
            );
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": false,
                        "unsupported": true,
                        "path": path,
                        "message": msg,
                    })
                );
            } else {
                println!("UNSUPPORTED: {msg}");
            }
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub enum AdapterToolAction {
    New { name: String },
    Validate { path: PathBuf },
}

pub fn cmd_analyze(
    json: bool,
    path: PathBuf,
    to: Option<String>,
    write_lock: bool,
) -> Result<(), ParallaxError> {
    let reg = builtin_registry();
    let analysis = analyze_stack(&path, &reg, to.as_deref())?;
    if write_lock {
        let lock = AdapterLockfile::from_registry(&reg);
        let out = path.join("parallax.lock");
        lock.write(&out)?;
        if !json {
            println!("Wrote {}", out.display());
        }
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&analysis).unwrap());
        return Ok(());
    }
    println!("Detected stack ({})", analysis.root);
    println!("Project type: {}", analysis.project_kind);
    if !analysis.language_mix.is_empty() {
        println!("\nLanguage mix:");
        let mut mix: Vec<_> = analysis.language_mix.iter().collect();
        mix.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
        for (lang, pct) in mix {
            println!("  {lang:16} {pct:5.1}%");
        }
    }
    println!("\nDetected:");
    for d in &analysis.detected {
        println!(
            "  [{:^16}] {} ({}) — {}",
            d.adapter_type,
            d.name,
            d.maturity,
            d.detection.confidence.as_str()
        );
    }
    println!("\nSelected migration adapters:");
    for s in &analysis.stack.selected {
        println!("  {}  ({})", s.adapter_id, s.role);
    }
    if !analysis.stack.conflicts_resolved.is_empty() {
        println!("\nConflicts resolved:");
        for c in &analysis.stack.conflicts_resolved {
            println!("  {c}");
        }
    }
    if let Some(t) = &analysis.stack.target_suggestion {
        println!("\nTarget suggestion (--to {}):", t.language);
        println!("  pair maturity: {}", t.pair_maturity);
        if let Some(fw) = &t.framework {
            println!("  framework:     {fw}");
        }
        if let Some(orm) = &t.orm {
            println!("  orm:           {orm}");
        }
        if let Some(rt) = &t.async_runtime {
            println!("  async:         {rt}");
        }
        for r in &t.rationale {
            println!("  · {r}");
        }
    }
    let e = &analysis.completeness_estimate;
    println!("\nEstimated translation:");
    println!("  Exact...............{}%", e.exact_pct);
    println!("  High confidence.....{}%", e.high_confidence_pct);
    println!("  Review..............{}%", e.review_pct);
    println!("  Unsupported.........{}%", e.unsupported_pct);
    println!("  ({})", e.notes);
    if !analysis.unsupported.is_empty() {
        println!("\nAdapter limitations:");
        for u in &analysis.unsupported {
            println!("  · {u}");
        }
    }
    Ok(())
}

pub fn cmd_stacks(json: bool) -> Result<(), ParallaxError> {
    let presets = stack_presets();
    if json {
        println!("{}", serde_json::to_string_pretty(&presets).unwrap());
        return Ok(());
    }
    println!("Stack presets\n");
    for p in presets {
        println!(
            "{:16} → {} ({})",
            p["id"].as_str().unwrap_or("?"),
            p["components"]
                .as_array()
                .map(|a| a
                    .iter()
                    .filter_map(|x| x.as_str())
                    .collect::<Vec<_>>()
                    .join(" + "))
                .unwrap_or_default(),
            p["maturity"].as_str().unwrap_or("")
        );
    }
    println!("\nUse: plx migrate . --to rust   (auto-selects from source behavior)");
    println!("     plx analyze . --to rust   (preview selected stack)");
    Ok(())
}

fn stack_presets() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({"id":"rust-api","maturity":"stable","components":["rust","axum","tokio","serde","sqlx","tracing","cargo"]}),
        serde_json::json!({"id":"rust-cli","maturity":"beta","components":["rust","clap","tokio","cargo"]}),
        serde_json::json!({"id":"rust-library","maturity":"stable","components":["rust","cargo","serde"]}),
        serde_json::json!({"id":"go-api","maturity":"beta","components":["go","chi","database/sql","go modules"]}),
        serde_json::json!({"id":"python-api","maturity":"beta","components":["python","fastapi","pydantic","uvicorn","pytest"]}),
        serde_json::json!({"id":"java-service","maturity":"experimental","components":["java","spring-boot","maven"]}),
        serde_json::json!({"id":"dotnet-api","maturity":"experimental","components":["csharp","aspnet","efcore","msbuild"]}),
    ]
}

pub fn cmd_mappings(json: bool, query: Option<String>) -> Result<(), ParallaxError> {
    let kb = DependencyMapDb::builtin();
    let keys = kb.list_keys();
    let filtered: Vec<_> = if let Some(q) = query {
        let q = q.to_ascii_lowercase();
        keys.into_iter()
            .filter(|p| p.to_ascii_lowercase().contains(&q))
            .collect()
    } else {
        keys
    };
    if json {
        let rows: Vec<_> = filtered.iter().filter_map(|name| kb.lookup(name)).collect();
        println!("{}", serde_json::to_string_pretty(&rows).unwrap());
        return Ok(());
    }
    println!("Dependency mappings ({})\n", filtered.len());
    for name in filtered.iter().take(80) {
        if let Some(entry) = kb.lookup(name) {
            let targets: Vec<String> = entry
                .equivalents
                .iter()
                .map(|e| format!("{}:{} ({:.0}%)", e.ecosystem, e.name, e.confidence * 100.0))
                .collect();
            println!(
                "{}:{}  →  {}",
                entry.ecosystem,
                entry.name,
                targets.join(", ")
            );
        }
    }
    if filtered.len() > 80 {
        println!("… {} more (use --json or a query)", filtered.len() - 80);
    }
    Ok(())
}

pub fn cmd_compatibility(json: bool, source: String, target: String) -> Result<(), ParallaxError> {
    let report = pair_compatibility(&source, &target);
    if json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
        return Ok(());
    }
    println!("{} → {}", report.source, report.target);
    for f in &report.features {
        let dots = ".".repeat((22usize).saturating_sub(f.feature.len()).max(2));
        println!("{}{}{}%", f.feature, dots, f.score_pct);
    }
    println!(
        "\nOverall: {} ({}%)",
        report.overall.to_uppercase(),
        report.overall_pct
    );
    Ok(())
}

pub fn cmd_unsupported(json: bool, path: PathBuf) -> Result<(), ParallaxError> {
    let reg = builtin_registry();
    let analysis = analyze_stack(&path, &reg, None)?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "root": analysis.root,
                "unsupported": analysis.unsupported,
                "scaffold_adapters": analysis.detected.iter().filter(|d| d.maturity == "scaffold" || d.maturity == "parse_only").map(|d| &d.id).collect::<Vec<_>>(),
            }))
            .unwrap()
        );
        return Ok(());
    }
    println!("Unsupported / limited surface ({})\n", analysis.root);
    if analysis.unsupported.is_empty() {
        println!("No scaffold-only adapters in the selected stack.");
        println!("(Deep construct-level Unsupported requires full analyze/migrate.)");
    } else {
        for u in &analysis.unsupported {
            println!("· {u}");
        }
    }
    Ok(())
}

pub fn cmd_explain_stack(
    json: bool,
    path: PathBuf,
    to: Option<String>,
) -> Result<(), ParallaxError> {
    let reg = builtin_registry();
    let analysis = analyze_stack(&path, &reg, to.as_deref())?;
    let suggestion = analysis.stack.target_suggestion.clone();
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "analysis_selected": analysis.stack.selected,
                "suggestion": suggestion,
                "conflicts": analysis.stack.conflicts_resolved,
            }))
            .unwrap()
        );
        return Ok(());
    }
    println!("Explain stack ({})\n", analysis.root);
    if let Some(t) = suggestion {
        if let Some(fw) = &t.framework {
            println!("Why {fw}?");
        } else {
            println!("Target language: {}", t.language);
        }
        println!("Pair maturity: {}", t.pair_maturity);
        for r in &t.rationale {
            println!("  · {r}");
        }
        if let Some(orm) = &t.orm {
            println!("\nORM: {orm}");
        }
        if let Some(rt) = &t.async_runtime {
            println!("Async runtime: {rt}");
        }
    } else {
        println!("Pass --to <language> for target stack rationale.");
        println!("Selected source adapters:");
        for s in &analysis.stack.selected {
            println!("  {} ({})", s.adapter_id, s.role);
        }
    }
    Ok(())
}

fn short_adapter_label(id: &str) -> String {
    let s = id.strip_prefix("parallax.").unwrap_or(id);
    let parts: Vec<_> = s.split('.').collect();
    match parts.as_slice() {
        [lang, "source" | "target"] => (*lang).to_string(),
        [_, name] => (*name).to_string(),
        [_, _, name] => (*name).to_string(),
        _ => s.rsplit('.').next().unwrap_or(s).to_string(),
    }
}

fn print_grouped(list: &[parallax_adapter_sdk::AdapterManifest]) {
    let groups = [
        ("LANGUAGE ADAPTERS", AdapterKind::SourceLanguage),
        ("TARGET ADAPTERS", AdapterKind::TargetLanguage),
        ("FRAMEWORK ADAPTERS", AdapterKind::Framework),
        ("WEB FRONTEND", AdapterKind::WebFrontend),
        ("BUILD ADAPTERS", AdapterKind::BuildSystem),
        ("TEST ADAPTERS", AdapterKind::TestFramework),
        ("DATABASE", AdapterKind::Database),
        ("ORM ADAPTERS", AdapterKind::Orm),
        ("DEPLOYMENT", AdapterKind::Deployment),
        ("RUNTIME", AdapterKind::Runtime),
        ("CLI FRAMEWORKS", AdapterKind::CliFramework),
        ("VALIDATION / SERIALIZATION", AdapterKind::Validation),
        ("FORMATTERS", AdapterKind::Formatter),
        ("LINTERS", AdapterKind::Linter),
        ("CODEGEN", AdapterKind::Codegen),
        ("DESKTOP GUI", AdapterKind::DesktopGui),
        ("PAIR PROFILES", AdapterKind::PairProfile),
    ];
    for (title, kind) in groups {
        let rows: Vec<_> = list.iter().filter(|m| m.adapter_type == kind).collect();
        if rows.is_empty() {
            continue;
        }
        println!("{title}");
        for m in rows {
            let short = short_adapter_label(m.id.as_str());
            println!("  {short:20} {}", m.maturity.as_str());
        }
        println!();
    }
    // Serialization kind (separate from Validation group title above)
    let serde: Vec<_> = list
        .iter()
        .filter(|m| m.adapter_type == AdapterKind::Serialization)
        .collect();
    if !serde.is_empty() {
        println!("SERIALIZATION");
        for m in serde {
            let short = short_adapter_label(m.id.as_str());
            println!("  {short:20} {}", m.maturity.as_str());
        }
        println!();
    }
    let other: Vec<_> = list
        .iter()
        .filter(|m| {
            !matches!(
                m.adapter_type,
                AdapterKind::SourceLanguage
                    | AdapterKind::TargetLanguage
                    | AdapterKind::Framework
                    | AdapterKind::WebFrontend
                    | AdapterKind::BuildSystem
                    | AdapterKind::TestFramework
                    | AdapterKind::Database
                    | AdapterKind::Orm
                    | AdapterKind::Deployment
                    | AdapterKind::Runtime
                    | AdapterKind::CliFramework
                    | AdapterKind::Validation
                    | AdapterKind::Serialization
                    | AdapterKind::Formatter
                    | AdapterKind::Linter
                    | AdapterKind::Codegen
                    | AdapterKind::DesktopGui
                    | AdapterKind::PairProfile
            )
        })
        .collect();
    if !other.is_empty() {
        println!("OTHER");
        for m in other {
            println!(
                "  {:28} {}",
                short_adapter_label(m.id.as_str()),
                m.maturity.as_str()
            );
        }
    }
}
