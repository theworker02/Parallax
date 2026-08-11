//! Structured tracing and host diagnostics for Parallax.

#![deny(unsafe_code)]
#![warn(missing_docs)]

use parallax_core::{ComponentVersions, RuntimeKind, RuntimeStatus};
use serde::{Deserialize, Serialize};
use tracing_subscriber::{fmt, EnvFilter};

/// Initialize tracing from `RUST_LOG` / explicit filter.
pub fn init_tracing(json: bool, verbose: bool) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if verbose {
            EnvFilter::new("info,parallax=debug")
        } else {
            EnvFilter::new("warn,parallax=info")
        }
    });
    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_target(true)
        .with_writer(std::io::stderr);
    if json {
        let _ = tracing::subscriber::set_global_default(subscriber.json().finish());
    } else {
        let _ = tracing::subscriber::set_global_default(subscriber.finish());
    }
}

/// Single runtime health check result.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeHealth {
    /// Runtime kind.
    pub runtime: RuntimeKind,
    /// Status.
    pub status: RuntimeStatus,
    /// Detected binary/path if any.
    pub binary: Option<String>,
    /// Version string if probed.
    pub version: Option<String>,
    /// Detail / error text.
    pub detail: Option<String>,
}

/// Full `plx doctor` report.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DoctorReport {
    /// Parallax version.
    pub parallax_version: String,
    /// Independently bumpable component versions.
    #[serde(default = "ComponentVersions::current")]
    pub versions: ComponentVersions,
    /// Host OS description.
    pub host: String,
    /// Per-runtime health.
    pub runtimes: Vec<RuntimeHealth>,
    /// Overall ok when at least one executable runtime is Ready.
    pub ok: bool,
}

impl DoctorReport {
    /// Format a human-readable report.
    pub fn format_human(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("Parallax {}\n", self.parallax_version));
        out.push_str(&format!(
            "Components: pir={} protocol={} snapshot={} adapter_interface={}\n",
            self.versions.pir_schema,
            self.versions.protocol,
            self.versions.snapshot,
            self.versions.adapter_interface
        ));
        out.push_str(&format!("Host: {}\n\n", self.host));
        for r in &self.runtimes {
            let status = match &r.status {
                RuntimeStatus::Ready => "READY".to_string(),
                RuntimeStatus::Degraded { reason } => format!("DEGRADED ({reason})"),
                RuntimeStatus::Unavailable { reason } => format!("UNAVAILABLE ({reason})"),
            };
            out.push_str(&format!("[{}] {}\n", r.runtime, status));
            if let Some(b) = &r.binary {
                out.push_str(&format!("  binary: {b}\n"));
            }
            if let Some(v) = &r.version {
                out.push_str(&format!("  version: {v}\n"));
            }
            if let Some(d) = &r.detail {
                out.push_str(&format!("  detail: {d}\n"));
            }
            out.push('\n');
        }
        if self.ok {
            out.push_str("Doctor: OK — at least one runtime is ready.\n");
        } else {
            out.push_str("Doctor: FAILED — no ready runtimes detected.\n");
        }
        out
    }
}

/// Trace event recorded during migrate/run for `--trace` / JSON output.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraceEvent {
    /// Monotonic order.
    pub seq: u64,
    /// Phase name.
    pub phase: String,
    /// Duration microseconds for this phase (if completed).
    pub duration_us: Option<u64>,
    /// Optional message.
    pub message: Option<String>,
}

/// Simple in-memory trace buffer.
#[derive(Clone, Debug, Default)]
pub struct TraceBuffer {
    events: Vec<TraceEvent>,
}

impl TraceBuffer {
    /// Create empty buffer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Push an event.
    pub fn push(
        &mut self,
        phase: impl Into<String>,
        duration_us: Option<u64>,
        message: Option<String>,
    ) {
        let seq = self.events.len() as u64;
        self.events.push(TraceEvent {
            seq,
            phase: phase.into(),
            duration_us,
            message,
        });
    }

    /// Borrow events.
    pub fn events(&self) -> &[TraceEvent] {
        &self.events
    }
}
