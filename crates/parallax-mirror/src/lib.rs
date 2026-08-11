//! Parallax Mirror — continuous cross-language synchronization.
//!
//! Links a source project to a target project via a semantic model, then
//! incrementally syncs changes without full remigration.

#![deny(unsafe_code)]
#![allow(missing_docs)]

mod diff;
mod differential;
mod history;
mod identity;
mod link;
mod merge;
mod ownership;
mod policy;
mod status;
mod sync;

pub use diff::{ChangeKind, SemanticChange, SemanticDiff};
pub use differential::{DifferentialCase, DifferentialResult, DifferentialRunner};
pub use history::{HistoryEntry, SyncHistory};
pub use identity::SemanticId;
pub use link::{LinkedProject, SemanticMapping, LINK_DIR};
pub use merge::{MergeDecision, SemanticConflict, SemanticMerge};
pub use ownership::{ManualRegion, RegionKind, RegionOwnership};
pub use policy::{LinkPolicy, SyncPolicy};
pub use status::{DriftKind, LinkStatus, SemanticDrift};
pub use sync::{sync_check, sync_link, SyncOptions, SyncReport};

use link::LinkedProject as Link;
use parallax_core::{ErrorCode, ParallaxError, Remediation};
use parallax_project::{SourceLanguage, TargetLanguage};
use std::path::{Path, PathBuf};

/// Create a link between source and target projects.
pub async fn link_projects(
    source: &Path,
    target: &Path,
    policy: SyncPolicy,
) -> Result<LinkedProject, ParallaxError> {
    Link::create(source, target, policy).await
}

/// Load an existing link from either project root (or explicit `.parallax-link`).
pub fn load_link(path: &Path) -> Result<LinkedProject, ParallaxError> {
    Link::load(path)
}

/// High-level status for CLI / JSON.
pub async fn link_status(path: &Path) -> Result<LinkStatus, ParallaxError> {
    let link = load_link(path)?;
    status::compute_status(&link).await
}

/// Explain a generated location using link source maps + semantic map.
pub fn explain(path: &Path, location: &str) -> Result<ExplainReport, ParallaxError> {
    let link = load_link(path)?;
    let target_root = PathBuf::from(&link.target_root);
    let entry = parallax_transmute::lookup_origin(&target_root, location).map_err(|e| {
        e.remediate(Remediation::new(
            "Ensure the target was produced by Transmute/Mirror and contains .plxmap.json",
        ))
    })?;
    let node = link
        .semantic_map
        .iter()
        .find(|m| {
            m.target_file.as_deref() == Some(entry.generated_file.as_str())
                || entry.semantic_node.contains(&m.qualified_name)
        })
        .cloned();
    Ok(ExplainReport {
        generated_file: entry.generated_file,
        generated_line: entry.generated_line,
        original_file: entry.original_file,
        original_line: entry.original_line,
        semantic_node: entry.semantic_node,
        mapping: node,
        confidence: "HIGH".into(),
    })
}

/// Why a target file changed (last sync history + semantic map).
pub fn why(path: &Path, target_file: &str) -> Result<WhyReport, ParallaxError> {
    let link = load_link(path)?;
    let hist = SyncHistory::load(&link.link_dir)?;
    let last = hist.entries.last().cloned();
    let related: Vec<_> = link
        .semantic_map
        .iter()
        .filter(|m| {
            m.target_file
                .as_ref()
                .map(|f| f.replace('\\', "/") == target_file.replace('\\', "/")
                    || target_file.ends_with(f.as_str()))
                .unwrap_or(false)
        })
        .cloned()
        .collect();
    Ok(WhyReport {
        target_file: target_file.into(),
        last_sync: last,
        related_nodes: related,
    })
}

/// Rollback to previous sync snapshot if available.
pub fn rollback(path: &Path) -> Result<String, ParallaxError> {
    history::rollback_last(path)
}

/// Machine-readable explain output.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ExplainReport {
    pub generated_file: String,
    pub generated_line: u32,
    pub original_file: String,
    pub original_line: u32,
    pub semantic_node: String,
    pub mapping: Option<link::SemanticMapping>,
    pub confidence: String,
}

/// Why-report.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WhyReport {
    pub target_file: String,
    pub last_sync: Option<HistoryEntry>,
    pub related_nodes: Vec<link::SemanticMapping>,
}

/// Pair quality tier (honest, based on implemented coverage).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PairTier {
    Tier1,
    Tier2,
    Experimental,
    Unsupported,
}

/// Look up language-pair maturity (delegates to connector catalog).
pub fn pair_tier(source: &SourceLanguage, target: &TargetLanguage) -> PairTier {
    use parallax_connectors::PairMaturity;
    match parallax_connectors::pair_maturity(source.as_str(), target.as_str()) {
        PairMaturity::Tier1 => PairTier::Tier1,
        PairMaturity::Tier2 => PairTier::Tier2,
        PairMaturity::Experimental => PairTier::Experimental,
        // Scaffold pairs are catalogued but not Mirror-ready yet.
        PairMaturity::Scaffold | PairMaturity::Unsupported => PairTier::Unsupported,
    }
}

pub(crate) fn io_err(e: std::io::Error) -> ParallaxError {
    ParallaxError::new(ErrorCode::Io, e.to_string()).with_source("parallax-mirror")
}
