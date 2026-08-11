//! CLI for Parallax Mirror.

use parallax_core::{ErrorCode, ParallaxError, Remediation};
use parallax_mirror::{
    explain, link_projects, link_status, load_link, rollback, sync_check, sync_link, why,
    DifferentialRunner, SyncHistory, SyncOptions, SyncPolicy,
};
use std::path::PathBuf;

pub async fn cmd_link(
    json: bool,
    source: PathBuf,
    target: PathBuf,
    policy: String,
) -> Result<(), ParallaxError> {
    let policy = SyncPolicy::parse(&policy).ok_or_else(|| {
        ParallaxError::new(
            ErrorCode::InvalidArgument,
            format!("unknown policy: {policy}"),
        )
        .remediate(Remediation::new(
            "Use source-authoritative|target-authoritative|bidirectional|manual",
        ))
    })?;
    let link = link_projects(&source, &target, policy).await?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "source": link.source_root,
                "target": link.target_root,
                "policy": link.policy.as_str(),
                "tier": link.pair_tier,
                "nodes": link.semantic_map.len(),
                "link_dir": link.link_dir,
            }))
            .unwrap()
        );
    } else {
        println!(" PARALLAX MIRROR LINK");
        println!("Source:  {}", link.source_root);
        println!("Target:  {}", link.target_root);
        println!("Policy:  {}", link.policy.as_str());
        println!("Tier:    {}", link.pair_tier);
        println!("Nodes:   {}", link.semantic_map.len());
        println!("Link:    {}", link.link_dir.display());
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn cmd_sync(
    json: bool,
    path: PathBuf,
    check: bool,
    reverse: bool,
    lint: bool,
    patch: bool,
    no_verify: bool,
    property: bool,
    deterministic: bool,
) -> Result<(), ParallaxError> {
    let opts = SyncOptions {
        check_only: check,
        reverse,
        lint,
        patch,
        verify: !no_verify && !check,
        deterministic,
        property,
        performance: false,
        since: None,
    };
    let report = if check {
        sync_check(&path).await?
    } else {
        sync_link(&path, &opts).await?
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        println!(" PARALLAX MIRROR");
        if check {
            println!("{}", report.message);
            if report.out_of_date {
                println!("Semantic changes:");
                for s in &report.change_summaries {
                    println!("  {s}");
                }
            }
        } else {
            println!("Source changes detected:");
            for s in &report.change_summaries {
                println!("  {s}");
            }
            if report.changes == 0 {
                println!("  (none)");
            }
            println!(
                "Regenerating..............{}",
                if report.files_touched.is_empty() && report.changes > 0 {
                    "partial"
                } else {
                    "done"
                }
            );
            if let Some(b) = report.build_ok {
                println!(
                    "Building..................{}",
                    if b { "done" } else { "failed" }
                );
            }
            if let Some(t) = report.tests_ok {
                println!(
                    "Running tests.............{}",
                    if t { "done" } else { "failed" }
                );
            }
            println!("Differential verification.done");
            println!("{}", report.message);
            println!("Files touched:");
            println!("{}", report.files_touched.len());
            for f in &report.files_touched {
                println!("  {f}");
            }
            for c in &report.conflicts {
                println!("! {c}");
            }
        }
    }
    if check && report.out_of_date {
        return Err(
            ParallaxError::new(ErrorCode::MigrationRejected, "link out of date")
                .with_source("parallax-cli")
                .with_operation("sync --check"),
        );
    }
    if !check && (report.build_ok == Some(false) || report.tests_ok == Some(false)) {
        return Err(ParallaxError::new(
            ErrorCode::MigrationRejected,
            "sync verification failed",
        ));
    }
    Ok(())
}

pub async fn cmd_status(json: bool, path: PathBuf) -> Result<(), ParallaxError> {
    let st = link_status(&path).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&st).unwrap());
    } else {
        println!("PARALLAX LINK STATUS");
        println!(
            "Source....................{}",
            if st.source_clean { "clean" } else { "changed" }
        );
        println!(
            "Target....................{}",
            if st.target_modified {
                "modified"
            } else {
                "clean"
            }
        );
        println!(
            "Synchronization...........behind by {} nodes",
            st.nodes_behind
        );
        println!(
            "Behavior..................{} mismatch(es)",
            st.manual_conflicts
        );
        println!("Dependencies...............see dependency-map.json");
        println!(
            "Status:\n{}",
            if st.attention_required {
                "ATTENTION REQUIRED"
            } else {
                "OK"
            }
        );
    }
    Ok(())
}

pub async fn cmd_ci(json: bool, path: PathBuf) -> Result<(), ParallaxError> {
    cmd_sync(
        json,
        path.clone(),
        true,
        false,
        false,
        false,
        false,
        false,
        false,
    )
    .await?;
    let link = load_link(&path)?;
    let results = DifferentialRunner::verify_target_tests(std::path::Path::new(&link.target_root))?;
    if json {
        println!("{}", serde_json::to_string_pretty(&results).unwrap());
    } else {
        println!("plx ci: sync --check OK");
        for r in &results {
            println!(
                "verify: {} — {}",
                r.name,
                if r.matched { "PASS" } else { "FAIL" }
            );
        }
    }
    if results.iter().any(|r| !r.matched) {
        return Err(ParallaxError::new(
            ErrorCode::MigrationRejected,
            "ci verification failed",
        ));
    }
    Ok(())
}

pub fn cmd_history(json: bool, path: PathBuf) -> Result<(), ParallaxError> {
    let link = load_link(&path)?;
    let hist = SyncHistory::load(&link.link_dir)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&hist).unwrap());
    } else {
        println!("PARALLAX SYNC HISTORY");
        for (i, e) in hist.entries.iter().enumerate() {
            println!(
                "#{i} {} changes={} files={} {}",
                e.at,
                e.semantic_changes,
                e.files_touched.len(),
                e.verification
            );
        }
        if hist.entries.is_empty() {
            println!("(empty)");
        }
    }
    Ok(())
}

pub fn cmd_rollback(json: bool, path: PathBuf) -> Result<(), ParallaxError> {
    let msg = rollback(&path)?;
    if json {
        println!(
            "{{\"ok\":true,\"message\":{}}}",
            serde_json::to_string(&msg).unwrap()
        );
    } else {
        println!("{msg}");
    }
    Ok(())
}

pub fn cmd_explain(json: bool, path: PathBuf, location: String) -> Result<(), ParallaxError> {
    let r = explain(&path, &location)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&r).unwrap());
    } else {
        println!("Generated from:");
        println!("{}:{}", r.original_file, r.original_line);
        println!("Semantic node:");
        println!("{}", r.semantic_node);
        println!("Confidence:");
        println!("{}", r.confidence);
    }
    Ok(())
}

pub fn cmd_why(json: bool, path: PathBuf, file: String) -> Result<(), ParallaxError> {
    let r = why(&path, &file)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&r).unwrap());
    } else {
        println!("Target file: {}", r.target_file);
        if let Some(last) = &r.last_sync {
            println!("Last sync: {}", last.at);
            println!("Semantic changes: {}", last.semantic_changes);
            println!("Verification: {}", last.verification);
        }
        println!("Related nodes:");
        for n in &r.related_nodes {
            println!("  {} ({})", n.qualified_name, n.id);
        }
    }
    Ok(())
}

pub async fn cmd_verify(
    json: bool,
    path: PathBuf,
    property: bool,
    deterministic: bool,
    performance: bool,
) -> Result<(), ParallaxError> {
    let link = load_link(&path)?;
    let results = DifferentialRunner::verify_target_tests(std::path::Path::new(&link.target_root))?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "results": results,
                "property": property,
                "deterministic": deterministic,
                "performance": performance,
                "note": "Property/performance modes report testing confidence, not formal proof",
            }))
            .unwrap()
        );
    } else {
        println!(" PARALLAX VERIFY");
        for r in &results {
            println!(
                "{}: {}",
                r.name,
                if r.matched { "MATCH" } else { "MISMATCH" }
            );
            println!("  {}", r.detail);
        }
        if property {
            println!("Note: --property increases testing confidence; it is not a formal proof.");
        }
    }
    if results.iter().any(|r| !r.matched) {
        return Err(ParallaxError::new(
            ErrorCode::MigrationRejected,
            "verification mismatch",
        ));
    }
    Ok(())
}
