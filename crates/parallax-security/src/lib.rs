//! Capability tokens and resource-limit policies for Parallax.

#![deny(unsafe_code)]
#![warn(missing_docs)]

use parallax_core::{ExecutionLimits, Remediation};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Sandbox / security policy applied to guest executions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SandboxPolicy {
    /// Execution limits.
    pub limits: ExecutionLimits,
    /// Whether guest code may perform network I/O (not enforced in MVP workers).
    pub allow_network: bool,
    /// Whether guest code may read arbitrary filesystem paths.
    pub allow_fs_read: bool,
    /// Whether guest code may write to the filesystem.
    pub allow_fs_write: bool,
    /// Maximum concurrent workers managed by the runtime.
    pub max_concurrent_workers: usize,
}

impl Default for SandboxPolicy {
    fn default() -> Self {
        Self {
            limits: ExecutionLimits::default(),
            allow_network: false,
            allow_fs_read: true,
            allow_fs_write: false,
            max_concurrent_workers: 4,
        }
    }
}

impl SandboxPolicy {
    /// Strict policy for untrusted code.
    pub fn strict() -> Self {
        Self {
            limits: ExecutionLimits {
                timeout: Duration::from_secs(5),
                max_output_bytes: 256 * 1024,
                max_message_bytes: 4 * 1024 * 1024,
                max_memory_bytes: Some(64 * 1024 * 1024),
                max_fuel: Some(1_000_000),
            },
            allow_network: false,
            allow_fs_read: false,
            allow_fs_write: false,
            max_concurrent_workers: 2,
        }
    }

    /// Validate that configured limits are sane.
    pub fn validate(&self) -> Result<(), String> {
        if self.limits.timeout.is_zero() {
            return Err("timeout must be > 0".into());
        }
        if self.max_concurrent_workers == 0 {
            return Err("max_concurrent_workers must be >= 1".into());
        }
        Ok(())
    }
}

/// Granted capability token recorded in snapshots / state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityToken {
    /// Token name.
    pub name: String,
    /// Optional scope description.
    pub scope: Option<String>,
}

impl CapabilityToken {
    /// Construct a named token.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            scope: None,
        }
    }
}

/// Helper remediations for common security violations.
pub fn remediation_for_network() -> Remediation {
    Remediation::with_detail(
        "Disable network-dependent guest code or raise allow_network in policy",
        "MVP adapters do not grant network access by default",
    )
}
