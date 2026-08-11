//! Explicit runtime capability declarations.
//!
//! Adapters must never hide limitations. Every capture/restore feature is
//! reported with a clear support level.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Support level for a single capability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityLevel {
    /// Fully supported with well-defined semantics.
    Yes,
    /// Partially supported; see adapter docs for constraints.
    Partial,
    /// Experimental and subject to change or failure.
    Experimental,
    /// Not supported. Attempts return `Unsupported`.
    No,
}

impl CapabilityLevel {
    /// Whether the capability may be attempted.
    pub fn is_usable(self) -> bool {
        matches!(self, Self::Yes | Self::Partial | Self::Experimental)
    }

    /// Compact status glyph for CLI tables.
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Yes => "YES",
            Self::Partial => "PARTIAL",
            Self::Experimental => "EXPERIMENTAL",
            Self::No => "NO",
        }
    }
}

impl fmt::Display for CapabilityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.glyph())
    }
}

/// Declared capabilities of a runtime adapter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeCapabilities {
    /// Convert guest values to/from PIR.
    pub values: CapabilityLevel,
    /// Capture/restore global bindings.
    pub globals: CapabilityLevel,
    /// Capture/restore local bindings.
    pub locals: CapabilityLevel,
    /// Represent functions as references.
    pub functions: CapabilityLevel,
    /// Capture closures with environment.
    pub closures: CapabilityLevel,
    /// Capture call-stack frames (alias surface: stack_frames).
    pub stack_capture: CapabilityLevel,
    /// Structured stack frame export into UniversalFrame / UES.
    #[serde(default = "default_no")]
    pub stack_frames: CapabilityLevel,
    /// Control / instruction position at safepoints.
    #[serde(default = "default_no")]
    pub control_position: CapabilityLevel,
    /// In-flight exception state capture/restore.
    #[serde(default = "default_no")]
    pub exception_state: CapabilityLevel,
    /// Capture a continuation / UES at a supported safepoint.
    #[serde(default = "default_no")]
    pub continuation_capture: CapabilityLevel,
    /// Restore and continue from captured continuations.
    pub continuation_restore: CapabilityLevel,
    /// Resume a continuation on a different runtime.
    #[serde(default = "default_no")]
    pub cross_runtime_resume: CapabilityLevel,
    /// Migrate in-flight async tasks.
    pub async_migration: CapabilityLevel,
    /// Execute guest programs.
    pub execution: CapabilityLevel,
    /// Capture stdout/stderr.
    pub stdio_capture: CapabilityLevel,
    /// Enforce execution timeouts.
    pub timeouts: CapabilityLevel,
    /// Enforce memory/output limits.
    pub resource_limits: CapabilityLevel,
    /// Cancel in-flight work.
    pub cancellation: CapabilityLevel,
}

fn default_no() -> CapabilityLevel {
    CapabilityLevel::No
}

impl RuntimeCapabilities {
    /// Fully unsupported baseline.
    pub fn none() -> Self {
        Self {
            values: CapabilityLevel::No,
            globals: CapabilityLevel::No,
            locals: CapabilityLevel::No,
            functions: CapabilityLevel::No,
            closures: CapabilityLevel::No,
            stack_capture: CapabilityLevel::No,
            stack_frames: CapabilityLevel::No,
            control_position: CapabilityLevel::No,
            exception_state: CapabilityLevel::No,
            continuation_capture: CapabilityLevel::No,
            continuation_restore: CapabilityLevel::No,
            cross_runtime_resume: CapabilityLevel::No,
            async_migration: CapabilityLevel::No,
            execution: CapabilityLevel::No,
            stdio_capture: CapabilityLevel::No,
            timeouts: CapabilityLevel::No,
            resource_limits: CapabilityLevel::No,
            cancellation: CapabilityLevel::No,
        }
    }

    /// Python adapter capability profile (Continuum: explicit checkpoint EXP).
    pub fn python() -> Self {
        Self {
            values: CapabilityLevel::Yes,
            globals: CapabilityLevel::Yes,
            locals: CapabilityLevel::Partial,
            functions: CapabilityLevel::Partial,
            closures: CapabilityLevel::Partial,
            stack_capture: CapabilityLevel::Experimental,
            stack_frames: CapabilityLevel::Experimental,
            control_position: CapabilityLevel::Experimental,
            exception_state: CapabilityLevel::No,
            continuation_capture: CapabilityLevel::Experimental,
            continuation_restore: CapabilityLevel::Experimental,
            cross_runtime_resume: CapabilityLevel::No,
            async_migration: CapabilityLevel::No,
            execution: CapabilityLevel::Yes,
            stdio_capture: CapabilityLevel::Yes,
            timeouts: CapabilityLevel::Yes,
            resource_limits: CapabilityLevel::Partial,
            cancellation: CapabilityLevel::Yes,
        }
    }

    /// JavaScript adapter capability profile.
    pub fn javascript() -> Self {
        Self {
            values: CapabilityLevel::Yes,
            globals: CapabilityLevel::Yes,
            locals: CapabilityLevel::Partial,
            functions: CapabilityLevel::Partial,
            closures: CapabilityLevel::Partial,
            stack_capture: CapabilityLevel::No,
            stack_frames: CapabilityLevel::Experimental,
            control_position: CapabilityLevel::Experimental,
            exception_state: CapabilityLevel::No,
            continuation_capture: CapabilityLevel::Experimental,
            continuation_restore: CapabilityLevel::Experimental,
            cross_runtime_resume: CapabilityLevel::No,
            async_migration: CapabilityLevel::No,
            execution: CapabilityLevel::Yes,
            stdio_capture: CapabilityLevel::Yes,
            timeouts: CapabilityLevel::Yes,
            resource_limits: CapabilityLevel::Partial,
            cancellation: CapabilityLevel::Yes,
        }
    }

    /// WASM adapter capability profile — deliberately constrained.
    pub fn wasm() -> Self {
        Self {
            values: CapabilityLevel::Partial,
            globals: CapabilityLevel::No,
            locals: CapabilityLevel::No,
            functions: CapabilityLevel::Partial,
            closures: CapabilityLevel::No,
            stack_capture: CapabilityLevel::No,
            stack_frames: CapabilityLevel::No,
            control_position: CapabilityLevel::No,
            exception_state: CapabilityLevel::No,
            continuation_capture: CapabilityLevel::No,
            continuation_restore: CapabilityLevel::No,
            cross_runtime_resume: CapabilityLevel::No,
            async_migration: CapabilityLevel::No,
            execution: CapabilityLevel::Yes,
            stdio_capture: CapabilityLevel::No,
            timeouts: CapabilityLevel::Yes,
            resource_limits: CapabilityLevel::Yes,
            cancellation: CapabilityLevel::Yes,
        }
    }

    /// Iterate named capabilities for CLI display (value + continuum fields).
    pub fn entries(&self) -> Vec<(&'static str, CapabilityLevel)> {
        vec![
            ("Values", self.values),
            ("Globals", self.globals),
            ("Locals", self.locals),
            ("Functions", self.functions),
            ("Closures", self.closures),
            ("Stack capture", self.stack_capture),
            ("Stack frames", self.stack_frames),
            ("Control position", self.control_position),
            ("Exception state", self.exception_state),
            ("Continuation capture", self.continuation_capture),
            ("Continuation restore", self.continuation_restore),
            ("Cross-runtime resume", self.cross_runtime_resume),
            ("Async migration", self.async_migration),
            ("Execution", self.execution),
            ("Stdio capture", self.stdio_capture),
            ("Timeouts", self.timeouts),
            ("Resource limits", self.resource_limits),
            ("Cancellation", self.cancellation),
        ]
    }

    /// Continuation-only rows for `plx capabilities --continuations`.
    pub fn continuation_entries(&self) -> Vec<(&'static str, CapabilityLevel)> {
        vec![
            ("Stack frames", self.stack_frames),
            ("Control position", self.control_position),
            ("Exception state", self.exception_state),
            ("Closures", self.closures),
            ("Async migration", self.async_migration),
            ("Continuation capture", self.continuation_capture),
            ("Continuation restore", self.continuation_restore),
            ("Cross-runtime resume", self.cross_runtime_resume),
        ]
    }

    /// Look up a capability by canonical snake_case name.
    pub fn level_of(&self, name: &str) -> Option<CapabilityLevel> {
        match name {
            "values" => Some(self.values),
            "globals" => Some(self.globals),
            "locals" => Some(self.locals),
            "functions" => Some(self.functions),
            "closures" => Some(self.closures),
            "stack_capture" => Some(self.stack_capture),
            "stack_frames" => Some(self.stack_frames),
            "control_position" => Some(self.control_position),
            "exception_state" => Some(self.exception_state),
            "continuation_capture" => Some(self.continuation_capture),
            "continuation_restore" => Some(self.continuation_restore),
            "cross_runtime_resume" => Some(self.cross_runtime_resume),
            "async_migration" => Some(self.async_migration),
            "execution" => Some(self.execution),
            "stdio_capture" => Some(self.stdio_capture),
            "timeouts" => Some(self.timeouts),
            "resource_limits" => Some(self.resource_limits),
            "cancellation" => Some(self.cancellation),
            _ => None,
        }
    }

    /// Require a usable capability or return a structured `CapabilityViolation`.
    pub fn require(
        &self,
        name: &str,
        runtime: &crate::runtime::RuntimeKind,
    ) -> crate::Result<()> {
        let level = self.level_of(name).unwrap_or(CapabilityLevel::No);
        if level.is_usable() {
            return Ok(());
        }
        Err(crate::error::ParallaxError::new(
            crate::error::ErrorCode::CapabilityViolation,
            format!("capability '{name}' is unsupported on {runtime}"),
        )
        .with_source("parallax-core")
        .with_operation("require_capability")
        .with_runtime(runtime.clone())
        .context("capability", name.to_string())
        .context("level", level.glyph())
        .remediate(crate::error::Remediation::new(
            "Choose a runtime that declares this capability, or omit the unsupported operation",
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_can_execute() {
        assert!(RuntimeCapabilities::python().execution.is_usable());
        assert!(RuntimeCapabilities::python()
            .continuation_capture
            .is_usable());
        assert!(!RuntimeCapabilities::python()
            .cross_runtime_resume
            .is_usable());
    }

    #[test]
    fn wasm_has_no_continuations() {
        let w = RuntimeCapabilities::wasm();
        assert!(!w.continuation_capture.is_usable());
        assert!(!w.continuation_restore.is_usable());
    }
}
