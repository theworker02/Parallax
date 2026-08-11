//! Honest scaffold RuntimeAdapter for catalog languages.

use crate::catalog::{ConnectorDef, ConnectorMaturity};
use async_trait::async_trait;
use parallax_core::{
    CapabilityLevel, ErrorCode, ExecutionRequest, ExecutionResult, ParallaxError, Remediation,
    RestoreResult, RuntimeCapabilities, RuntimeKind, RuntimeMetadata, RuntimeStatus,
    PARALLAX_VERSION,
};
use parallax_ir::PirDocument;
use parallax_runtime::{BoxRuntimeAdapter, RuntimeAdapter};
use parallax_security::SandboxPolicy;
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;

struct HostProbe {
    path: Option<PathBuf>,
    version: Option<String>,
}

/// Scaffold adapter: probes host tools, refuses execute/restore until implemented.
pub struct ScaffoldAdapter {
    def: &'static ConnectorDef,
    host: Mutex<Option<HostProbe>>,
}

impl ScaffoldAdapter {
    pub fn new(def: &'static ConnectorDef) -> Self {
        Self {
            def,
            host: Mutex::new(None),
        }
    }

    pub fn boxed(def: &'static ConnectorDef) -> BoxRuntimeAdapter {
        Arc::new(Self::new(def))
    }

    fn kind_val(&self) -> RuntimeKind {
        RuntimeKind::Other(self.def.id.to_string())
    }

    fn ensure_host(&self) -> HostProbe {
        let mut g = self.host.lock();
        if let Some(h) = g.as_ref() {
            return HostProbe {
                path: h.path.clone(),
                version: h.version.clone(),
            };
        }
        let (path, version) = probe_host(self.def.host_binaries);
        let h = HostProbe { path, version };
        *g = Some(HostProbe {
            path: h.path.clone(),
            version: h.version.clone(),
        });
        h
    }

    fn unsupported(&self, op: &str) -> ParallaxError {
        ParallaxError::new(
            ErrorCode::UnsupportedValue,
            format!(
                "connector '{}' is {} — {op} is not implemented yet",
                self.def.id,
                self.def.maturity.as_str()
            ),
        )
        .with_runtime(self.kind_val())
        .with_source("parallax-connectors")
        .with_operation(op)
        .remediate(Remediation::new(format!(
            "{} — contribute a worker under adapters/{}/ or raise maturity past scaffold. See `plx connectors {}`",
            self.def.notes, self.def.id, self.def.id
        )))
    }
}

fn probe_host(bins: &[&str]) -> (Option<PathBuf>, Option<String>) {
    // Existence-only probe — never spawn `--version` (dozens of connectors would hang `plx runtimes`).
    for bin in bins {
        if let Some(path) = find_on_path(bin) {
            return (Some(path), None);
        }
    }
    (None, None)
}

fn find_on_path(bin: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(bin);
        if candidate.is_file() {
            return Some(candidate);
        }
        #[cfg(windows)]
        {
            for ext in ["exe", "cmd", "bat", "ps1"] {
                let c = dir.join(format!("{bin}.{ext}"));
                if c.is_file() {
                    return Some(c);
                }
            }
        }
    }
    None
}

fn scaffold_capabilities(def: &ConnectorDef) -> RuntimeCapabilities {
    let mut caps = RuntimeCapabilities::none();
    match def.maturity {
        ConnectorMaturity::Experimental => {
            caps.execution = CapabilityLevel::Experimental;
            caps.timeouts = CapabilityLevel::Experimental;
        }
        ConnectorMaturity::Scaffold
        | ConnectorMaturity::Planned
        | ConnectorMaturity::Production => {
            caps.execution = CapabilityLevel::No;
        }
    }
    caps
}

#[async_trait]
impl RuntimeAdapter for ScaffoldAdapter {
    fn kind(&self) -> RuntimeKind {
        self.kind_val()
    }

    fn metadata(&self) -> RuntimeMetadata {
        let host = self.ensure_host();
        let mut meta = RuntimeMetadata::builtin(
            self.kind_val(),
            self.def.name,
            format!("[{}] {}", self.def.maturity.as_str(), self.def.notes),
        );
        meta.adapter_version = PARALLAX_VERSION.to_string();
        meta.host_version = host.version;
        meta
    }

    fn capabilities(&self) -> RuntimeCapabilities {
        scaffold_capabilities(self.def)
    }

    async fn probe(&self) -> RuntimeStatus {
        if self.def.host_binaries.is_empty() {
            return RuntimeStatus::Unavailable {
                reason: format!(
                    "scaffold connector '{}' has no host binary (identity only)",
                    self.def.id
                ),
            };
        }
        let host = self.ensure_host();
        match host.path {
            Some(p) => RuntimeStatus::Degraded {
                reason: format!(
                    "host found at {} but connector is {} — execute/restore Unsupported",
                    p.display(),
                    self.def.maturity.as_str()
                ),
            },
            None => RuntimeStatus::Unavailable {
                reason: format!(
                    "host tool not found (tried: {}); connector is {}",
                    self.def.host_binaries.join(", "),
                    self.def.maturity.as_str()
                ),
            },
        }
    }

    async fn execute(
        &self,
        _request: ExecutionRequest,
        _policy: &SandboxPolicy,
    ) -> parallax_core::Result<ExecutionResult> {
        Err(self.unsupported("execute"))
    }

    async fn restore(
        &self,
        _pir: &PirDocument,
        _policy: &SandboxPolicy,
    ) -> parallax_core::Result<RestoreResult> {
        Err(self.unsupported("restore"))
    }
}
