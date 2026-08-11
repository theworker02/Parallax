//! Parallax Adapter SDK — extension surface for Atlas.
//!
//! Core orchestrates adapters; language/framework logic lives in adapters.

#![deny(unsafe_code)]
#![allow(missing_docs)]

mod capabilities;
mod detect;
mod manifest;
mod traits;

pub use capabilities::{
    AdapterCapabilities, CapabilityFlag, CapabilitySupport, ConstructCapability,
};
pub use detect::{DetectionConfidence, DetectionEvidence, DetectionResult, ProjectContext};
pub use manifest::{
    AdapterAuthor, AdapterId, AdapterKind, AdapterManifest, AdapterMaturity, AdapterPermissions,
    ConformanceLevel,
};
pub use traits::{
    BuildSystemAdapter, ConfigurationAdapter, DatabaseAdapter, DependencyAdapter,
    DeploymentAdapter, FrameworkAdapter, ParallaxAdapter, SourceLanguageAdapter,
    TargetLanguageAdapter, TestFrameworkAdapter, VerificationAdapter,
};

/// SDK schema version (independent of product SemVer).
pub const ADAPTER_SDK_VERSION: u32 = 1;
