//! `.parallax/` migration workspace for resume / inspect.

use crate::plan::MigrationPlan;
use crate::report::TransmuteReport;
use parallax_core::{ErrorCode, ParallaxError};
use parallax_project::ProjectAnalysis;
use std::fs;
use std::path::{Path, PathBuf};

/// On-disk workspace under the source project.
pub struct TransmuteWorkspace {
    /// Root `.parallax` directory.
    pub dir: PathBuf,
}

impl TransmuteWorkspace {
    /// Create / refresh workspace from analysis.
    pub fn create(source_root: &Path, analysis: &ProjectAnalysis, plan: &MigrationPlan) -> Result<Self, ParallaxError> {
        let dir = source_root.join(".parallax");
        fs::create_dir_all(dir.join("cache")).map_err(|e| {
            ParallaxError::new(ErrorCode::Io, e.to_string()).with_source("parallax-transmute")
        })?;
        let project = serde_json::json!({
            "root": analysis.root,
            "language": analysis.primary_language.to_string(),
            "framework": analysis.framework,
            "analyzed_at": analysis.analyzed_at,
        });
        fs::write(dir.join("project.json"), serde_json::to_string_pretty(&project).unwrap())?;
        fs::write(
            dir.join("graph.json"),
            serde_json::to_string_pretty(&analysis.graph).map_err(|e| {
                ParallaxError::new(ErrorCode::SerializationFailure, e.to_string())
            })?,
        )?;
        fs::write(
            dir.join("puir.json"),
            serde_json::to_string_pretty(&analysis.puir).map_err(|e| {
                ParallaxError::new(ErrorCode::SerializationFailure, e.to_string())
            })?,
        )?;
        fs::write(
            dir.join("mappings.json"),
            serde_json::to_string_pretty(&plan.dependencies).unwrap_or_default(),
        )?;
        let _ = plan;
        Ok(Self { dir })
    }

    /// Write plan.json.
    pub fn write_plan(&self, plan: &MigrationPlan) -> Result<(), ParallaxError> {
        fs::write(
            self.dir.join("plan.json"),
            serde_json::to_string_pretty(plan).map_err(|e| {
                ParallaxError::new(ErrorCode::SerializationFailure, e.to_string())
            })?,
        )?;
        Ok(())
    }

    /// Write diagnostics.
    pub fn write_diagnostics(&self, report: &TransmuteReport) -> Result<(), ParallaxError> {
        fs::write(
            self.dir.join("diagnostics.json"),
            serde_json::to_string_pretty(&report.manual_reviews).unwrap_or_default(),
        )?;
        Ok(())
    }
}
