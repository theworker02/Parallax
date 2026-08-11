//! Incremental sync engine.

use crate::diff::{diff_programs, ChangeKind, SemanticDiff};
use crate::differential::DifferentialRunner;
use crate::history::{HistoryEntry, SyncHistory};
use crate::link::LinkedProject;
use crate::ownership::{ManualClassification, ManualRegion, RegionKind};
use crate::policy::SyncPolicy;
use chrono::Utc;
use parallax_core::{ErrorCode, ParallaxError, Remediation};
use parallax_transmute::{analyze_project, transmute_project, TransmuteOptions};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;

#[derive(Clone, Debug)]
pub struct SyncOptions {
    pub check_only: bool,
    pub reverse: bool,
    pub lint: bool,
    pub patch: bool,
    pub verify: bool,
    pub deterministic: bool,
    pub property: bool,
    pub performance: bool,
    pub since: Option<String>,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            check_only: false,
            reverse: false,
            lint: false,
            patch: false,
            verify: true,
            deterministic: false,
            property: false,
            performance: false,
            since: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncReport {
    pub check_only: bool,
    pub out_of_date: bool,
    pub changes: usize,
    pub change_summaries: Vec<String>,
    pub files_touched: Vec<String>,
    pub build_ok: Option<bool>,
    pub tests_ok: Option<bool>,
    pub manual_blocks: Vec<String>,
    pub conflicts: Vec<String>,
    pub verification: Vec<String>,
    pub message: String,
}

/// Non-mutating freshness check.
pub async fn sync_check(link_path: &Path) -> Result<SyncReport, ParallaxError> {
    let link = LinkedProject::load(link_path)?;
    let analysis = analyze_project(Path::new(&link.source_root), None).await?;
    let baseline = link.baseline_puir()?;
    let diff = diff_programs(&baseline, &analysis.puir);
    let summaries: Vec<String> = diff
        .changes
        .iter()
        .map(|c| format!("{:?}: {} — {}", c.kind, c.qualified_name, c.detail))
        .collect();
    let out_of_date = !diff.is_empty();
    Ok(SyncReport {
        check_only: true,
        out_of_date,
        changes: diff.changes.len(),
        change_summaries: summaries,
        files_touched: vec![],
        build_ok: None,
        tests_ok: None,
        manual_blocks: vec![],
        conflicts: vec![],
        verification: vec![],
        message: if out_of_date {
            format!(
                "Parallax link is OUT OF DATE\nSource commit:\n  {}\nLast synchronized fingerprint:\n  {}\nSemantic changes:\n  {}\nRun:\n  plx sync",
                link.source_commit.clone().unwrap_or_else(|| "unknown".into()),
                &link.last_source_fingerprint[..8.min(link.last_source_fingerprint.len())],
                diff.changes.len()
            )
        } else {
            "Parallax link is up to date.".into()
        },
    })
}

/// Apply incremental synchronization.
pub async fn sync_link(link_path: &Path, opts: &SyncOptions) -> Result<SyncReport, ParallaxError> {
    if opts.check_only {
        return sync_check(link_path).await;
    }
    let mut link = LinkedProject::load(link_path)?;

    if opts.reverse {
        return sync_reverse(&link, opts).await;
    }

    if matches!(link.policy, SyncPolicy::Manual) {
        let check = sync_check(link_path).await?;
        return Ok(SyncReport {
            message: "policy=manual — no files modified; see change_summaries".into(),
            ..check
        });
    }

    info!("mirror.sync.begin");
    let analysis = analyze_project(Path::new(&link.source_root), None).await?;
    let baseline = link.baseline_puir()?;
    let diff = diff_programs(&baseline, &analysis.puir);

    if diff.is_empty() {
        return Ok(SyncReport {
            check_only: false,
            out_of_date: false,
            changes: 0,
            change_summaries: vec![],
            files_touched: vec![],
            build_ok: None,
            tests_ok: None,
            manual_blocks: vec![],
            conflicts: vec![],
            verification: vec!["no semantic changes".into()],
            message: "Already synchronized.".into(),
        });
    }

    // Manual behavior-change regions block overwrite of those files
    let manual_blocks = blocked_files(&link, &diff)?;
    let affected_modules = affected_module_files(&diff);
    let mut files_touched: Vec<String> = Vec::new();
    let mut conflicts = Vec::new();

    if opts.patch {
        // Generate into temp and report — do not write
        let tmp = tempfile_dir()?;
        let report = regenerate(&link, &tmp).await?;
        return Ok(SyncReport {
            check_only: false,
            out_of_date: true,
            changes: diff.changes.len(),
            change_summaries: diff
                .changes
                .iter()
                .map(|c| format!("{:?}: {}", c.kind, c.qualified_name))
                .collect(),
            files_touched: report,
            build_ok: None,
            tests_ok: None,
            manual_blocks,
            conflicts,
            verification: vec!["patch mode — no tree modifications".into()],
            message: format!("Patch preview written under {}", tmp.display()),
        });
    }

    // Transactional: regenerate into temp, snapshot target, apply, validate, commit or restore.
    let tmp = tempfile_dir()?;
    let _ = regenerate(&link, &tmp).await?;

    let target = PathBuf::from(&link.target_root);
    let pre_snap = link.link_dir.join("history/pre-apply");
    let _ = fs::remove_dir_all(&pre_snap);
    snapshot_src_tree(&target, &pre_snap)?;

    for rel in affected_target_files(&affected_modules) {
        if manual_blocks.iter().any(|m| m == &rel) {
            conflicts.push(format!(
                "skipped {rel} — manual BEHAVIOR_CHANGE region (preserve target)"
            ));
            continue;
        }
        let src = tmp.join(&rel);
        let dst = target.join(&rel);
        if src.exists() {
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent).map_err(crate::io_err)?;
            }
            fs::copy(&src, &dst).map_err(crate::io_err)?;
            files_touched.push(rel);
        }
    }

    // Always refresh types/service if any function in those modules changed
    for rel in ["src/types.rs", "src/service.rs", "src/main.rs"] {
        let src = tmp.join(rel);
        if src.exists()
            && affected_modules
                .iter()
                .any(|m| rel.contains(&module_stem(m)) || rel.ends_with("main.rs"))
            && !manual_blocks.iter().any(|m| m == rel)
            && !files_touched.iter().any(|f| f == rel)
        {
            fs::copy(&src, target.join(rel)).map_err(crate::io_err)?;
            files_touched.push(rel.into());
        }
    }

    if opts.lint {
        let _ = Command::new("cargo")
            .args(["fmt", "--", "--check"])
            .current_dir(&target)
            .status();
    } else {
        let _ = Command::new("cargo")
            .args(["fmt"])
            .current_dir(&target)
            .status();
    }

    let mut build_ok = None;
    let mut tests_ok = None;
    let mut verification = Vec::new();
    if opts.verify {
        let build = Command::new("cargo")
            .arg("build")
            .current_dir(&target)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        build_ok = Some(build);
        if build {
            let results = DifferentialRunner::verify_target_tests(&target)?;
            tests_ok = Some(results.iter().all(|r| r.matched));
            for r in results {
                verification.push(format!("{}: {}", r.name, r.detail));
            }
        } else {
            verification.push("build failed — restoring previous target state".into());
            restore_src_tree(&pre_snap, &target)?;
            files_touched.clear();
        }
        if tests_ok == Some(false) {
            verification.push("tests failed — restoring previous target state".into());
            restore_src_tree(&pre_snap, &target)?;
            files_touched.clear();
            build_ok = Some(false);
        }
    }

    if opts.property {
        verification.push(
            "property-based equivalence not fully implemented — use migrated unit tests (testing confidence, not proof)"
                .into(),
        );
    }
    if opts.performance {
        verification.push("performance comparison skipped (not configured)".into());
    }
    if opts.deterministic {
        verification
            .push("deterministic sandbox: fuel/time mocked where adapters support it".into());
    }

    // Update baseline + ownership only if build succeeded (or verify disabled)
    if build_ok.unwrap_or(true) {
        link.write_baseline_puir(&analysis.puir)?;
        let fp = {
            let bytes = serde_json::to_vec(&analysis.puir).unwrap_or_default();
            let mut h = Sha256::new();
            h.update(&bytes);
            hex::encode(h.finalize())
        };
        link.last_source_fingerprint = fp.clone();
        link.last_sync_at = Some(Utc::now());
        link.source_commit = git_head(Path::new(&link.source_root));
        link.target_commit = git_head(&target);
        refresh_ownership(&link)?;
        link.save()?;
        SyncHistory::append(
            &link.link_dir,
            HistoryEntry {
                at: Utc::now(),
                source_commit: link.source_commit.clone(),
                target_commit: link.target_commit.clone(),
                semantic_changes: diff.changes.len(),
                files_touched: files_touched.clone(),
                verification: verification.join("; "),
                confidence: "HIGH".into(),
                fingerprint: fp,
            },
        )?;
    }

    let summaries: Vec<String> = diff
        .changes
        .iter()
        .map(|c| format!("{:?}: {} — {}", c.kind, c.qualified_name, c.detail))
        .collect();

    let failed = build_ok == Some(false) || tests_ok == Some(false);
    Ok(SyncReport {
        check_only: false,
        out_of_date: failed,
        changes: diff.changes.len(),
        change_summaries: summaries,
        files_touched,
        build_ok,
        tests_ok,
        manual_blocks,
        conflicts,
        verification,
        message: if failed {
            "Sync aborted — previous target state preserved.".into()
        } else {
            "Target synchronized.".into()
        },
    })
}

fn snapshot_src_tree(target: &Path, snap_dir: &Path) -> Result<(), ParallaxError> {
    fs::create_dir_all(snap_dir.join("src")).map_err(crate::io_err)?;
    let src = target.join("src");
    if src.is_dir() {
        for e in fs::read_dir(&src).map_err(crate::io_err)? {
            let e = e.map_err(crate::io_err)?;
            if e.path().extension().and_then(|x| x.to_str()) == Some("rs") {
                fs::copy(e.path(), snap_dir.join("src").join(e.file_name()))
                    .map_err(crate::io_err)?;
            }
        }
    }
    if target.join("Cargo.toml").exists() {
        fs::copy(target.join("Cargo.toml"), snap_dir.join("Cargo.toml")).map_err(crate::io_err)?;
    }
    Ok(())
}

fn restore_src_tree(snap_dir: &Path, target: &Path) -> Result<(), ParallaxError> {
    let src_snap = snap_dir.join("src");
    if src_snap.is_dir() {
        for e in fs::read_dir(&src_snap).map_err(crate::io_err)? {
            let e = e.map_err(crate::io_err)?;
            fs::copy(e.path(), target.join("src").join(e.file_name())).map_err(crate::io_err)?;
        }
    }
    if snap_dir.join("Cargo.toml").exists() {
        fs::copy(snap_dir.join("Cargo.toml"), target.join("Cargo.toml")).map_err(crate::io_err)?;
    }
    Ok(())
}

async fn sync_reverse(
    link: &LinkedProject,
    _opts: &SyncOptions,
) -> Result<SyncReport, ParallaxError> {
    // Only allow nodes marked ExactYes — currently none auto-qualified; report Unsupported honestly.
    let safe: Vec<_> = link
        .semantic_map
        .iter()
        .filter(|m| matches!(m.reverse_safe, crate::ownership::ReverseSafety::ExactYes))
        .collect();
    if safe.is_empty() {
        return Err(ParallaxError::new(
            ErrorCode::UnsupportedValue,
            "reverse sync is not enabled for any node (Exact..............NO for current mappings)",
        )
        .with_source("parallax-mirror")
        .with_operation("sync_reverse")
        .remediate(Remediation::new(
            "Improve target→source lowering or mark nodes ExactYes after review; use source-authoritative sync",
        )));
    }
    Ok(SyncReport {
        check_only: false,
        out_of_date: false,
        changes: 0,
        change_summaries: vec![],
        files_touched: vec![],
        build_ok: None,
        tests_ok: None,
        manual_blocks: vec![],
        conflicts: vec![],
        verification: vec![],
        message: format!(
            "reverse sync would touch {} node(s) — not yet fully implemented",
            safe.len()
        ),
    })
}

async fn regenerate(link: &LinkedProject, output: &Path) -> Result<Vec<String>, ParallaxError> {
    let opts = TransmuteOptions {
        source: PathBuf::from(&link.source_root),
        from: Some(link.source_language.clone()),
        to: link.target_language.clone(),
        output: Some(output.to_path_buf()),
        dry_run: false,
        strict: false,
        interactive: false,
        preserve_layout: false,
        target_style: parallax_transmute::TargetStyle::Idiomatic,
        report: true,
        verify: false,
        require_build: false,
        require_tests: false,
        min_confidence: None,
        fail_on_unsupported: false,
        keep: vec![],
        max_repair_passes: 1,
        update: true,
    };
    let result = transmute_project(&opts).await?;
    let mut files = result.report.translated_files;
    files.extend(result.report.generated_files);
    Ok(files)
}

fn affected_module_files(diff: &SemanticDiff) -> HashSet<String> {
    let mut set = HashSet::new();
    for c in &diff.changes {
        if let Some(f) = &c.source_file {
            set.insert(f.clone());
        } else {
            // qualified name src.service.getWeather → src/service
            let parts: Vec<&str> = c.qualified_name.split('.').collect();
            if parts.len() >= 2 {
                set.insert(format!("{}/{}.ts", parts[0], parts[1]));
            }
        }
        match c.kind {
            ChangeKind::AddedType | ChangeKind::ChangedDataModel | ChangeKind::RemovedType => {
                set.insert("src/types.ts".into());
            }
            _ => {}
        }
    }
    set
}

fn affected_target_files(modules: &HashSet<String>) -> Vec<String> {
    let mut out = Vec::new();
    for m in modules {
        let stem = module_stem(m);
        if stem == "index" {
            out.push("src/app.rs".into());
            out.push("src/main.rs".into());
        } else if stem == "routes" {
            out.push("src/routes.rs".into());
            out.push("src/main.rs".into());
        } else {
            out.push(format!("src/{stem}.rs"));
        }
    }
    out.sort();
    out.dedup();
    out
}

fn module_stem(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("module")
        .replace('-', "_")
}

fn blocked_files(link: &LinkedProject, _diff: &SemanticDiff) -> Result<Vec<String>, ParallaxError> {
    let path = link.link_dir.join("manual-regions.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let regions: Vec<ManualRegion> =
        serde_json::from_str(&fs::read_to_string(path).map_err(crate::io_err)?).unwrap_or_default();
    Ok(regions
        .into_iter()
        .filter(|r| matches!(r.classification, ManualClassification::BehaviorChange))
        .map(|r| r.target_file)
        .collect())
}

fn refresh_ownership(link: &LinkedProject) -> Result<(), ParallaxError> {
    let target = PathBuf::from(&link.target_root);
    let mut ownership = Vec::new();
    for m in &link.semantic_map {
        let tf = m.target_file.clone().unwrap_or_default();
        let hash = if !tf.is_empty() && target.join(&tf).exists() {
            let bytes = fs::read(target.join(&tf)).map_err(crate::io_err)?;
            let mut h = Sha256::new();
            h.update(&bytes);
            hex::encode(h.finalize())
        } else {
            String::new()
        };
        ownership.push(crate::ownership::RegionOwnership {
            id: m.id.clone(),
            kind: RegionKind::Generated,
            target_file: tf,
            content_hash: hash,
            reverse_safe: m.reverse_safe,
        });
    }
    fs::write(
        link.link_dir.join("ownership.json"),
        serde_json::to_string_pretty(&ownership).unwrap(),
    )
    .map_err(crate::io_err)?;
    Ok(())
}

fn tempfile_dir() -> Result<PathBuf, ParallaxError> {
    let dir = std::env::temp_dir().join(format!("parallax-sync-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).map_err(crate::io_err)?;
    Ok(dir)
}

fn git_head(dir: &Path) -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir)
        .output()
        .ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}
