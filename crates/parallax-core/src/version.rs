//! Version constants for independently evolving Parallax interfaces.
//!
//! Each constant can be bumped without implying a product release bump.
//! Compatibility helpers fail clearly when a peer speaks a newer major.

use crate::error::{ErrorCode, ParallaxError, Remediation, Result};
use serde::{Deserialize, Serialize};

/// Parallax product version (`CARGO_PKG_VERSION` / workspace).
pub const PARALLAX_VERSION: &str = env!("CARGO_PKG_VERSION");

/// PIR schema version. Bump when the IR wire format changes incompatibly.
pub const PIR_SCHEMA_VERSION: u32 = 1;

/// Worker protocol version.
pub const PROTOCOL_VERSION: u32 = 1;

/// Snapshot (`.plx`) format version.
pub const SNAPSHOT_FORMAT_VERSION: u32 = 1;

/// Adapter interface version.
pub const ADAPTER_INTERFACE_VERSION: u32 = 1;

/// Universal Execution State (UES) format version.
/// Independent of PIR — UES models suspended execution, PIR models values.
pub const UES_FORMAT_VERSION: u32 = 1;

/// Parallax Continuation IR (PCIR) schema version.
/// Independent of PIR and UES.
pub const PCIR_SCHEMA_VERSION: u32 = 1;

/// Parallax Universal Program IR (PUIR) schema version.
/// Independent of PIR / UES / PCIR — PUIR models program semantics for Transmute.
pub const PUIR_SCHEMA_VERSION: u32 = 1;

/// Mirror link metadata format version (`.parallax-link/`).
pub const MIRROR_LINK_FORMAT_VERSION: u32 = 1;

/// Aggregate of independently bumpable component versions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentVersions {
    /// Product / workspace version string.
    pub parallax: String,
    /// PIR schema major.
    pub pir_schema: u32,
    /// Worker NDJSON protocol major.
    pub protocol: u32,
    /// Snapshot (`.plx`) format major.
    pub snapshot: u32,
    /// Adapter trait / capability surface major.
    pub adapter_interface: u32,
    /// Universal Execution State format major.
    pub ues_format: u32,
    /// Parallax Continuation IR schema major.
    pub pcir_schema: u32,
    /// Parallax Universal Program IR schema major.
    pub puir_schema: u32,
    /// Mirror link metadata format major.
    pub mirror_link_format: u32,
}

impl ComponentVersions {
    /// Versions compiled into this binary.
    pub fn current() -> Self {
        Self {
            parallax: PARALLAX_VERSION.to_string(),
            pir_schema: PIR_SCHEMA_VERSION,
            protocol: PROTOCOL_VERSION,
            snapshot: SNAPSHOT_FORMAT_VERSION,
            adapter_interface: ADAPTER_INTERFACE_VERSION,
            ues_format: UES_FORMAT_VERSION,
            pcir_schema: PCIR_SCHEMA_VERSION,
            puir_schema: PUIR_SCHEMA_VERSION,
            mirror_link_format: MIRROR_LINK_FORMAT_VERSION,
        }
    }

    /// Human-readable multi-line summary.
    pub fn format_human(&self) -> String {
        format!(
            "parallax {parallax}\n\
             pir_schema {pir}\n\
             protocol {proto}\n\
             snapshot {snap}\n\
             adapter_interface {iface}\n\
             ues_format {ues}\n\
             pcir_schema {pcir}\n\
             puir_schema {puir}\n\
             mirror_link_format {mirror}\n",
            parallax = self.parallax,
            pir = self.pir_schema,
            proto = self.protocol,
            snap = self.snapshot,
            iface = self.adapter_interface,
            ues = self.ues_format,
            pcir = self.pcir_schema,
            puir = self.puir_schema,
            mirror = self.mirror_link_format,
        )
    }
}

/// Reject unknown / future PIR schema majors.
pub fn check_pir_schema(version: u32) -> Result<()> {
    if version == 0 || version > PIR_SCHEMA_VERSION {
        return Err(ParallaxError::new(
            ErrorCode::InvalidSnapshot,
            format!(
                "unsupported PIR schema version {version} (supported: 1..={PIR_SCHEMA_VERSION})"
            ),
        )
        .with_source("parallax-core")
        .with_operation("check_pir_schema")
        .context("got", version.to_string())
        .context("expected_max", PIR_SCHEMA_VERSION.to_string())
        .remediate(Remediation::new(
            "Upgrade Parallax, or re-export the document with a supported PIR schema",
        )));
    }
    Ok(())
}

/// Reject unknown / future snapshot format majors.
pub fn check_snapshot_format(version: u32) -> Result<()> {
    if version != SNAPSHOT_FORMAT_VERSION {
        return Err(ParallaxError::new(
            ErrorCode::InvalidSnapshot,
            format!(
                "unsupported snapshot format version {version} (expected {SNAPSHOT_FORMAT_VERSION})"
            ),
        )
        .with_source("parallax-core")
        .with_operation("check_snapshot_format")
        .context("got", version.to_string())
        .context("expected", SNAPSHOT_FORMAT_VERSION.to_string())
        .remediate(Remediation::new(
            "Upgrade Parallax to read this snapshot, or recreate it with the current format",
        )));
    }
    Ok(())
}

/// Reject worker protocol version mismatches (exact match required).
pub fn check_protocol(version: u32) -> Result<()> {
    if version != PROTOCOL_VERSION {
        return Err(ParallaxError::new(
            ErrorCode::ProtocolViolation,
            format!("protocol version mismatch: got {version}, expected {PROTOCOL_VERSION}"),
        )
        .with_source("parallax-core")
        .with_operation("check_protocol")
        .context("got", version.to_string())
        .context("expected", PROTOCOL_VERSION.to_string())
        .remediate(Remediation::new(
            "Ensure host and worker were built from the same Parallax release",
        )));
    }
    Ok(())
}

/// Reject adapters targeting a newer interface than this host understands.
pub fn check_adapter_interface(version: u32) -> Result<()> {
    if version == 0 || version > ADAPTER_INTERFACE_VERSION {
        return Err(ParallaxError::new(
            ErrorCode::CapabilityViolation,
            format!(
                "unsupported adapter interface version {version} (supported: 1..={ADAPTER_INTERFACE_VERSION})"
            ),
        )
        .with_source("parallax-core")
        .with_operation("check_adapter_interface")
        .context("got", version.to_string())
        .context("expected_max", ADAPTER_INTERFACE_VERSION.to_string())
        .remediate(Remediation::new(
            "Upgrade the Parallax host or use an adapter built for this interface version",
        )));
    }
    Ok(())
}

/// Reject unknown / future UES format majors.
pub fn check_ues_format(version: u32) -> Result<()> {
    if version == 0 || version > UES_FORMAT_VERSION {
        return Err(ParallaxError::new(
            ErrorCode::InvalidSnapshot,
            format!(
                "unsupported UES format version {version} (supported: 1..={UES_FORMAT_VERSION})"
            ),
        )
        .with_source("parallax-core")
        .with_operation("check_ues_format")
        .context("got", version.to_string())
        .context("expected_max", UES_FORMAT_VERSION.to_string())
        .remediate(Remediation::new(
            "Upgrade Parallax, or recapture the continuation with a supported UES format",
        )));
    }
    Ok(())
}

/// Reject unknown / future PCIR schema majors.
pub fn check_pcir_schema(version: u32) -> Result<()> {
    if version == 0 || version > PCIR_SCHEMA_VERSION {
        return Err(ParallaxError::new(
            ErrorCode::InvalidSnapshot,
            format!(
                "unsupported PCIR schema version {version} (supported: 1..={PCIR_SCHEMA_VERSION})"
            ),
        )
        .with_source("parallax-core")
        .with_operation("check_pcir_schema")
        .context("got", version.to_string())
        .context("expected_max", PCIR_SCHEMA_VERSION.to_string())
        .remediate(Remediation::new(
            "Upgrade Parallax, or re-export the continuation with a supported PCIR schema",
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_versions_are_positive() {
        let v = ComponentVersions::current();
        assert_eq!(v.parallax, PARALLAX_VERSION);
        assert!(v.pir_schema >= 1);
        assert!(v.protocol >= 1);
        assert!(v.snapshot >= 1);
        assert!(v.adapter_interface >= 1);
        assert!(v.ues_format >= 1);
        assert!(v.pcir_schema >= 1);
        assert!(v.puir_schema >= 1);
        assert!(v.mirror_link_format >= 1);
    }

    #[test]
    fn rejects_future_ues_and_pcir() {
        assert_eq!(
            check_ues_format(UES_FORMAT_VERSION + 1).unwrap_err().code,
            ErrorCode::InvalidSnapshot
        );
        assert_eq!(
            check_pcir_schema(PCIR_SCHEMA_VERSION + 1)
                .unwrap_err()
                .code,
            ErrorCode::InvalidSnapshot
        );
        check_ues_format(UES_FORMAT_VERSION).unwrap();
        check_pcir_schema(PCIR_SCHEMA_VERSION).unwrap();
    }

    #[test]
    fn rejects_future_pir_schema() {
        let err = check_pir_schema(PIR_SCHEMA_VERSION + 1).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSnapshot);
        assert!(err.message.contains("unsupported PIR schema"));
    }

    #[test]
    fn rejects_zero_pir_schema() {
        assert!(check_pir_schema(0).is_err());
    }

    #[test]
    fn accepts_current_pir_schema() {
        check_pir_schema(PIR_SCHEMA_VERSION).unwrap();
    }

    #[test]
    fn rejects_future_snapshot_format() {
        let err = check_snapshot_format(SNAPSHOT_FORMAT_VERSION + 99).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSnapshot);
    }

    #[test]
    fn rejects_protocol_mismatch() {
        let err = check_protocol(PROTOCOL_VERSION + 1).unwrap_err();
        assert_eq!(err.code, ErrorCode::ProtocolViolation);
    }

    #[test]
    fn rejects_future_adapter_interface() {
        let err = check_adapter_interface(ADAPTER_INTERFACE_VERSION + 1).unwrap_err();
        assert_eq!(err.code, ErrorCode::CapabilityViolation);
    }

    #[test]
    fn component_versions_json_round_trip() {
        let v = ComponentVersions::current();
        let s = serde_json::to_string(&v).unwrap();
        let back: ComponentVersions = serde_json::from_str(&s).unwrap();
        assert_eq!(v, back);
        assert!(s.contains("adapter_interface"));
        assert!(s.contains("pir_schema"));
        assert!(s.contains("ues_format"));
        assert!(s.contains("pcir_schema"));
        assert!(s.contains("puir_schema"));
    }
}
