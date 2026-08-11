//! Language-neutral project semantic graph.

#![deny(unsafe_code)]
#![warn(missing_docs)]

mod graph;
mod lang;

pub use graph::{
    DependencyRef, Entrypoint, GraphEdge, GraphEdgeKind, GraphNode, GraphNodeKind, ProjectFile,
    ProjectGraph, ProjectPackage, Relationship,
};
pub use lang::{detect_languages, SourceLanguage, TargetLanguage};

use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use parallax_core::{ErrorCode, ParallaxError};
use parallax_puir::{PuirProgram, TypeInferenceBundle};
use serde::{Deserialize, Serialize};

/// Analysis snapshot used before migration planning.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectAnalysis {
    /// Root path (absolute).
    pub root: String,
    /// Detected primary source language.
    pub primary_language: SourceLanguage,
    /// Language mix percentages (0–100), measured from file counts.
    pub language_mix: IndexMap<String, f64>,
    /// Semantic graph.
    pub graph: ProjectGraph,
    /// Lowered PUIR program.
    pub puir: PuirProgram,
    /// Type inference bundle.
    pub types: TypeInferenceBundle,
    /// Detected framework if any (express, fastapi, …).
    pub framework: Option<String>,
    /// Detected database layer if any.
    pub database: Option<String>,
    /// When analysis ran.
    pub analyzed_at: DateTime<Utc>,
}

impl ProjectAnalysis {
    /// Validate embedded IR versions.
    pub fn validate(&self) -> Result<(), ParallaxError> {
        self.puir.validate()?;
        if self.graph.version == 0 {
            return Err(ParallaxError::new(
                ErrorCode::InvalidArgument,
                "project graph version 0 is invalid",
            )
            .with_source("parallax-project")
            .with_operation("validate"));
        }
        Ok(())
    }
}
