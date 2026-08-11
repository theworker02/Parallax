//! Deterministic `.plx` snapshot format.

#![deny(unsafe_code)]
#![warn(missing_docs)]

use chrono::{DateTime, Utc};
use parallax_core::{
    check_snapshot_format, ErrorCode, ExecutionState, ParallaxError, RuntimeKind, SnapshotId,
    SNAPSHOT_FORMAT_VERSION,
};
use parallax_ir::{content_hash, from_json_bytes, to_json_bytes_compact, PirDocument};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Magic bytes for `.plx` files (JSON document; magic in header field).
pub const SNAPSHOT_MAGIC: &str = "PARALLAX_PLX";

/// On-disk snapshot document.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    /// Magic marker.
    pub magic: String,
    /// Format version.
    pub format_version: u32,
    /// Snapshot id.
    pub id: SnapshotId,
    /// Creation time.
    pub created_at: DateTime<Utc>,
    /// Origin runtime.
    pub runtime: RuntimeKind,
    /// Optional label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Execution state shell.
    pub state: ExecutionState,
    /// PIR document (bindings / heap).
    pub pir: PirDocument,
    /// SHA-256 of canonical pir+state payload (excluding this field and id timestamps variance handled).
    pub content_hash: String,
}

impl Snapshot {
    /// Create a snapshot from state + PIR.
    pub fn create(
        runtime: RuntimeKind,
        state: ExecutionState,
        pir: PirDocument,
        label: Option<String>,
    ) -> parallax_core::Result<Self> {
        pir.validate()?;
        let mut snap = Self {
            magic: SNAPSHOT_MAGIC.into(),
            format_version: SNAPSHOT_FORMAT_VERSION,
            id: SnapshotId::new(),
            created_at: Utc::now(),
            runtime,
            label,
            state,
            pir,
            content_hash: String::new(),
        };
        snap.content_hash = snap.compute_hash()?;
        Ok(snap)
    }

    fn compute_hash(&self) -> parallax_core::Result<String> {
        let payload = serde_json::json!({
            "format_version": self.format_version,
            "runtime": self.runtime,
            "state": self.state,
            "pir": self.pir,
        });
        let bytes = serde_json::to_vec(&payload).map_err(ParallaxError::from)?;
        Ok(content_hash(&bytes))
    }

    /// Validate magic, version, PIR, and content hash.
    pub fn validate(&self) -> parallax_core::Result<()> {
        if self.magic != SNAPSHOT_MAGIC {
            return Err(ParallaxError::new(
                ErrorCode::InvalidSnapshot,
                format!("invalid snapshot magic: {}", self.magic),
            )
            .with_source("parallax-snapshot")
            .with_operation("validate"));
        }
        check_snapshot_format(self.format_version).map_err(|e| {
            e.with_source("parallax-snapshot")
                .with_operation("validate")
        })?;
        self.pir.validate()?;
        let expected = self.compute_hash()?;
        if expected != self.content_hash {
            return Err(ParallaxError::new(
                ErrorCode::InvalidSnapshot,
                "snapshot content hash mismatch",
            )
            .with_source("parallax-snapshot")
            .with_operation("validate")
            .context("expected", expected)
            .context("actual", self.content_hash.clone()));
        }
        Ok(())
    }

    /// Write snapshot to a `.plx` JSON file.
    pub fn write_to_path(&self, path: &Path) -> parallax_core::Result<()> {
        self.validate()?;
        let bytes = serde_json::to_vec_pretty(self).map_err(ParallaxError::from)?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path, bytes)?;
        Ok(())
    }

    /// Read and validate a snapshot from disk.
    pub fn read_from_path(path: &Path) -> parallax_core::Result<Self> {
        let bytes = fs::read(path)?;
        let snap: Self = serde_json::from_slice(&bytes).map_err(|e| {
            ParallaxError::new(ErrorCode::InvalidSnapshot, e.to_string())
                .with_source("parallax-snapshot")
                .with_operation("read_from_path")
        })?;
        snap.validate()?;
        Ok(snap)
    }

    /// Compact inspection summary.
    pub fn inspect_summary(&self) -> SnapshotInspect {
        SnapshotInspect {
            id: self.id.to_string(),
            runtime: self.runtime.clone(),
            created_at: self.created_at,
            label: self.label.clone(),
            format_version: self.format_version,
            content_hash: self.content_hash.clone(),
            binding_names: self.pir.bindings.keys().cloned().collect(),
            value_count: self.pir.value_count(),
            pir_schema: self.pir.schema,
        }
    }
}

/// Human/JSON inspect view.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SnapshotInspect {
    /// Snapshot id.
    pub id: String,
    /// Runtime.
    pub runtime: RuntimeKind,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Label.
    pub label: Option<String>,
    /// Format version.
    pub format_version: u32,
    /// Content hash.
    pub content_hash: String,
    /// Binding names.
    pub binding_names: Vec<String>,
    /// Value node count.
    pub value_count: usize,
    /// PIR schema.
    pub pir_schema: u32,
}

/// Encode PIR to bytes helper used by tests.
pub fn encode_pir(doc: &PirDocument) -> parallax_core::Result<Vec<u8>> {
    to_json_bytes_compact(doc)
}

/// Decode PIR helper.
pub fn decode_pir(bytes: &[u8]) -> parallax_core::Result<PirDocument> {
    from_json_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use parallax_core::{RuntimeCapabilities, RuntimeKind};
    use parallax_ir::{PirInteger, PirValue};
    use tempfile::tempdir;

    #[test]
    fn snapshot_round_trip() {
        let mut pir = PirDocument::new();
        pir.set_binding("state", PirValue::int_i64(42));
        let state = ExecutionState::empty(RuntimeKind::Python, RuntimeCapabilities::python());
        let snap = Snapshot::create(RuntimeKind::Python, state, pir, Some("t".into())).unwrap();
        let dir = tempdir().unwrap();
        let path = dir.path().join("t.plx");
        snap.write_to_path(&path).unwrap();
        let loaded = Snapshot::read_from_path(&path).unwrap();
        assert_eq!(loaded.content_hash, snap.content_hash);
        assert_eq!(
            loaded.pir.binding("state"),
            Some(&PirValue::Int {
                v: PirInteger::from_i64(42)
            })
        );
    }

    #[test]
    fn rejects_future_format_version() {
        let mut pir = PirDocument::new();
        pir.set_binding("state", PirValue::int_i64(1));
        let state = ExecutionState::empty(RuntimeKind::Python, RuntimeCapabilities::python());
        let mut snap = Snapshot::create(RuntimeKind::Python, state, pir, None).unwrap();
        snap.format_version = SNAPSHOT_FORMAT_VERSION + 1;
        let err = snap.validate().unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSnapshot);
        assert!(err.message.contains("unsupported snapshot format"));
    }
}
