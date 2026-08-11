//! Source maps (`.plxmap`) and `plx origin` lookup.

use parallax_core::{ErrorCode, ParallaxError, Remediation};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// One mapped region.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceMapEntry {
    /// Generated file (relative to output root).
    pub generated_file: String,
    /// Generated line (1-based).
    pub generated_line: u32,
    /// Optional generated column.
    pub generated_column: Option<u32>,
    /// Original file relative to source root.
    pub original_file: String,
    /// Original line.
    pub original_line: u32,
    /// Original column.
    pub original_column: u32,
    /// Semantic node description.
    pub semantic_node: String,
}

/// Source map file contents.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SourceMapFile {
    /// Version.
    pub version: u32,
    /// Entries.
    pub entries: Vec<SourceMapEntry>,
}

/// Look up origin for `file:line` in a migrated project.
pub fn lookup_origin(
    project_root: &Path,
    file_line: &str,
) -> Result<SourceMapEntry, ParallaxError> {
    let (file, line) = parse_file_line(file_line)?;
    let map_path = project_root.join(".plxmap.json");
    if !map_path.exists() {
        return Err(ParallaxError::new(
            ErrorCode::InvalidArgument,
            format!("no .plxmap.json in {}", project_root.display()),
        )
        .with_source("parallax-transmute")
        .with_operation("origin")
        .remediate(Remediation::new(
            "Run a Transmute migration that writes source maps",
        )));
    }
    let text = fs::read_to_string(&map_path)?;
    let map: SourceMapFile = serde_json::from_str(&text).map_err(|e| {
        ParallaxError::new(ErrorCode::SerializationFailure, e.to_string())
            .with_source("parallax-transmute")
    })?;
    map.entries
        .into_iter()
        .filter(|e| paths_match(&e.generated_file, &file) && e.generated_line == line)
        .max_by_key(|e| e.generated_line)
        .ok_or_else(|| {
            ParallaxError::new(
                ErrorCode::InvalidArgument,
                format!("no origin mapping for {file_line}"),
            )
            .with_source("parallax-transmute")
            .with_operation("origin")
        })
}

fn parse_file_line(s: &str) -> Result<(String, u32), ParallaxError> {
    let (file, line_s) = s.rsplit_once(':').ok_or_else(|| {
        ParallaxError::new(
            ErrorCode::InvalidArgument,
            "expected path:line (e.g. src/services/auth.rs:82)",
        )
        .with_source("parallax-transmute")
    })?;
    let line: u32 = line_s.parse().map_err(|_| {
        ParallaxError::new(ErrorCode::InvalidArgument, "invalid line number")
            .with_source("parallax-transmute")
    })?;
    Ok((file.to_string(), line))
}

fn paths_match(a: &str, b: &str) -> bool {
    let na = a.replace('\\', "/");
    let nb = b.replace('\\', "/");
    na == nb || na.ends_with(&nb) || nb.ends_with(&na)
}
