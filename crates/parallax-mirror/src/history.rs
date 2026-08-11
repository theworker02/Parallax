//! Sync history and rollback.

use chrono::{DateTime, Utc};
use parallax_core::{ErrorCode, ParallaxError, Remediation};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub at: DateTime<Utc>,
    pub source_commit: Option<String>,
    pub target_commit: Option<String>,
    pub semantic_changes: usize,
    pub files_touched: Vec<String>,
    pub verification: String,
    pub confidence: String,
    pub fingerprint: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SyncHistory {
    pub entries: Vec<HistoryEntry>,
}

impl SyncHistory {
    pub fn load(link_dir: &Path) -> Result<Self, ParallaxError> {
        let path = link_dir.join("history/log.json");
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = fs::read_to_string(path).map_err(crate::io_err)?;
        Ok(serde_json::from_str(&text).unwrap_or_default())
    }

    pub fn append(link_dir: &Path, entry: HistoryEntry) -> Result<(), ParallaxError> {
        fs::create_dir_all(link_dir.join("history")).map_err(crate::io_err)?;
        let mut hist = Self::load(link_dir)?;
        // snapshot target src before recording (for rollback)
        let snap_name = format!("snap-{}", entry.at.timestamp());
        let snap_dir = link_dir.join("history").join(&snap_name);
        let _ = snapshot_target(link_dir, &snap_dir);
        hist.entries.push(entry);
        fs::write(
            link_dir.join("history/log.json"),
            serde_json::to_string_pretty(&hist).unwrap(),
        )
        .map_err(crate::io_err)?;
        fs::write(link_dir.join("history/latest_snap"), snap_name).ok();
        Ok(())
    }
}

fn snapshot_target(link_dir: &Path, snap_dir: &Path) -> Result<(), ParallaxError> {
    let link = crate::link::LinkedProject::load(link_dir)?;
    let target = PathBuf::from(&link.target_root);
    fs::create_dir_all(snap_dir.join("src")).map_err(crate::io_err)?;
    let src = target.join("src");
    if src.is_dir() {
        for e in fs::read_dir(&src).map_err(crate::io_err)? {
            let e = e.map_err(crate::io_err)?;
            if e.path().extension().and_then(|x| x.to_str()) == Some("rs") {
                let dest = snap_dir.join("src").join(e.file_name());
                fs::copy(e.path(), dest).map_err(crate::io_err)?;
            }
        }
    }
    if target.join("Cargo.toml").exists() {
        fs::copy(target.join("Cargo.toml"), snap_dir.join("Cargo.toml")).map_err(crate::io_err)?;
    }
    Ok(())
}

pub fn rollback_last(path: &Path) -> Result<String, ParallaxError> {
    let link = crate::link::LinkedProject::load(path)?;
    let latest = fs::read_to_string(link.link_dir.join("history/latest_snap")).map_err(|_| {
        ParallaxError::new(ErrorCode::InvalidArgument, "no sync snapshot to roll back")
            .with_source("parallax-mirror")
            .remediate(Remediation::new("Run plx sync at least once"))
    })?;
    let snap = link.link_dir.join("history").join(latest.trim());
    if !snap.is_dir() {
        return Err(ParallaxError::new(
            ErrorCode::InvalidArgument,
            "snapshot directory missing",
        ));
    }
    let target = PathBuf::from(&link.target_root);
    let src_snap = snap.join("src");
    if src_snap.is_dir() {
        for e in fs::read_dir(&src_snap).map_err(crate::io_err)? {
            let e = e.map_err(crate::io_err)?;
            fs::copy(e.path(), target.join("src").join(e.file_name())).map_err(crate::io_err)?;
        }
    }
    if snap.join("Cargo.toml").exists() {
        fs::copy(snap.join("Cargo.toml"), target.join("Cargo.toml")).map_err(crate::io_err)?;
    }
    Ok(format!("Rolled back target from {}", snap.display()))
}
