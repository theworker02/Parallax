//! Adapter trait hierarchy.

use crate::capabilities::AdapterCapabilities;
use crate::detect::{DetectionResult, ProjectContext};
use crate::manifest::AdapterManifest;

/// Base adapter contract — every Atlas adapter implements this.
pub trait ParallaxAdapter: Send + Sync {
    fn manifest(&self) -> AdapterManifest;
    fn detect(&self, context: &ProjectContext) -> DetectionResult;
    fn capabilities(&self) -> AdapterCapabilities;
}

/// Source-language frontend (normalize to PUIR / ProjectGraph).
pub trait SourceLanguageAdapter: ParallaxAdapter {
    fn primary_language(&self) -> &str;
}

/// Target-language backend (consume PUIR + plan → idiomatic code).
pub trait TargetLanguageAdapter: ParallaxAdapter {
    fn primary_language(&self) -> &str;
    fn preferred_formatter(&self) -> Option<&str> {
        None
    }
}

/// Web / HTTP / application framework.
pub trait FrameworkAdapter: ParallaxAdapter {
    fn framework_id(&self) -> &str;
}

/// Package equivalence / capability matching.
pub trait DependencyAdapter: ParallaxAdapter {
    fn ecosystems(&self) -> &[&str];
}

pub trait BuildSystemAdapter: ParallaxAdapter {}
pub trait TestFrameworkAdapter: ParallaxAdapter {}
pub trait DatabaseAdapter: ParallaxAdapter {}
pub trait ConfigurationAdapter: ParallaxAdapter {}
pub trait DeploymentAdapter: ParallaxAdapter {}
pub trait VerificationAdapter: ParallaxAdapter {}
