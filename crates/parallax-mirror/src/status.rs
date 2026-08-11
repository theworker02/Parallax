//! Drift detection and `plx status`.

use crate::diff::diff_programs;
use crate::link::LinkedProject;
use crate::ownership::{ManualClassification, ManualRegion};
use chrono::{DateTime, Utc};
use parallax_core::ParallaxError;
use parallax_transmute::analyze_project;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

/// Drift category.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriftKind {
    SourceAhead,
    TargetAhead,
    BothChanged,
    DependencyDrift,
    BehaviorDrift,
    ConfigDrift,
    TestDrift,
}

/// One drift finding.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SemanticDrift {
    pub kind: DriftKind,
    pub detail: String,
}

/// Link status report.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LinkStatus {
    pub sync_status: String,
    pub behavior_score: f64,
    pub build_status: String,
    pub test_status: String,
    pub manual_conflicts: usize,
    pub unsupported_nodes: usize,
    pub source_commit: Option<String>,
    pub target_commit: Option<String>,
    pub source_clean: bool,
    pub target_modified: bool,
    pub nodes_behind: usize,
    pub drifts: Vec<SemanticDrift>,
    pub attention_required: bool,
    pub checked_at: DateTime<Utc>,
    pub pair_tier: String,
}

pub async fn compute_status(link: &LinkedProject) -> Result<LinkStatus, ParallaxError> {
    let analysis = analyze_project(PathBuf::from(&link.source_root).as_path(), None).await?;
    let baseline = link.baseline_puir()?;
    let diff = diff_programs(&baseline, &analysis.puir);
    let nodes_behind = diff.changes.len();

    let mut drifts = Vec::new();
    if nodes_behind > 0 {
        drifts.push(SemanticDrift {
            kind: DriftKind::SourceAhead,
            detail: format!("{nodes_behind} semantic node(s) changed since last sync"),
        });
    }

    let manual = load_manual(link)?;
    let behavior_conflicts = manual
        .iter()
        .filter(|m| matches!(m.classification, ManualClassification::BehaviorChange))
        .count();
    if behavior_conflicts > 0 {
        drifts.push(SemanticDrift {
            kind: DriftKind::BehaviorDrift,
            detail: format!("{behavior_conflicts} manual behavior-changing edit(s)"),
        });
    }

    let target_modified = detect_target_edits(link)?;
    if target_modified && nodes_behind > 0 {
        drifts.push(SemanticDrift {
            kind: DriftKind::BothChanged,
            detail: "source and target both modified".into(),
        });
    } else if target_modified {
        drifts.push(SemanticDrift {
            kind: DriftKind::TargetAhead,
            detail: "target files diverge from ownership hashes".into(),
        });
    }

    let sync_status = if nodes_behind == 0 && !target_modified {
        "in_sync"
    } else if nodes_behind > 0 {
        "out_of_date"
    } else {
        "target_modified"
    };

    Ok(LinkStatus {
        sync_status: sync_status.into(),
        behavior_score: if behavior_conflicts == 0 { 1.0 } else { 0.5 },
        build_status: "unknown".into(),
        test_status: "unknown".into(),
        manual_conflicts: behavior_conflicts,
        unsupported_nodes: 0,
        source_commit: link.source_commit.clone(),
        target_commit: link.target_commit.clone(),
        source_clean: nodes_behind == 0,
        target_modified,
        nodes_behind,
        drifts,
        attention_required: nodes_behind > 0 || behavior_conflicts > 0 || target_modified,
        checked_at: Utc::now(),
        pair_tier: link.pair_tier.clone(),
    })
}

fn load_manual(link: &LinkedProject) -> Result<Vec<ManualRegion>, ParallaxError> {
    let path = link.link_dir.join("manual-regions.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path).map_err(crate::io_err)?;
    Ok(serde_json::from_str(&text).unwrap_or_default())
}

fn detect_target_edits(link: &LinkedProject) -> Result<bool, ParallaxError> {
    let path = link.link_dir.join("ownership.json");
    if !path.exists() {
        return Ok(false);
    }
    let text = fs::read_to_string(&path).map_err(crate::io_err)?;
    let ownership: Vec<crate::ownership::RegionOwnership> =
        serde_json::from_str(&text).unwrap_or_default();
    let target = PathBuf::from(&link.target_root);
    for o in ownership {
        if o.target_file.is_empty() || o.content_hash.is_empty() {
            continue;
        }
        let p = target.join(&o.target_file);
        if !p.exists() {
            continue;
        }
        let bytes = fs::read(&p).map_err(crate::io_err)?;
        let mut h = Sha256::new();
        h.update(&bytes);
        let now = hex::encode(h.finalize());
        if now != o.content_hash {
            return Ok(true);
        }
    }
    Ok(false)
}
