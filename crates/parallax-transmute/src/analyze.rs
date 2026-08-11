//! Project analysis — inventory + frontend → ProjectAnalysis.

use crate::frontend::{run_typescript_frontend, is_project_root};
use crate::infer::infer_types;
use chrono::Utc;
use indexmap::IndexMap;
use parallax_core::{ErrorCode, ParallaxError, Remediation};
use parallax_project::{
    detect_languages, Entrypoint, GraphEdge, GraphEdgeKind, GraphNode, GraphNodeKind,
    ProjectAnalysis, ProjectFile, ProjectGraph, SourceLanguage,
};
use parallax_puir::{Module, PuirProgram, PUIR_SCHEMA_VERSION};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Analyze a project directory or single source file.
pub async fn analyze_project(
    root: &Path,
    from: Option<SourceLanguage>,
) -> Result<ProjectAnalysis, ParallaxError> {
    let root = if root.is_file() {
        root.parent().unwrap_or(Path::new(".")).to_path_buf()
    } else {
        root.to_path_buf()
    };
    let root = root.canonicalize().map_err(|e| {
        ParallaxError::new(ErrorCode::Io, e.to_string()).with_source("parallax-transmute")
    })?;

    if !is_project_root(&root) && !root.join("src").exists() {
        // still allow if there are .ts files
        let has_ts = WalkDir::new(&root)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
            .any(|e| {
                matches!(
                    e.path().extension().and_then(|x| x.to_str()),
                    Some("ts" | "js")
                )
            });
        if !has_ts {
            return Err(ParallaxError::new(
                ErrorCode::InvalidArgument,
                format!("not a recognizable project: {}", root.display()),
            )
            .with_source("parallax-transmute")
            .with_operation("analyze_project"));
        }
    }

    let files = inventory_files(&root)?;
    let paths: Vec<PathBuf> = files.iter().map(|f| PathBuf::from(&f.path)).collect();
    let (detected, mix) = detect_languages(&paths);
    let primary = from.or(detected).ok_or_else(|| {
        ParallaxError::new(
            ErrorCode::InvalidArgument,
            "could not detect source language; pass --from",
        )
        .with_source("parallax-transmute")
        .remediate(Remediation::new("Example: --from typescript"))
    })?;

    let mut language_mix = IndexMap::new();
    for (k, v) in mix {
        language_mix.insert(k, v);
    }

    let (mut graph, mut puir, framework, database) = match &primary {
        SourceLanguage::TypeScript | SourceLanguage::JavaScript => {
            let json = run_typescript_frontend(&root)?;
            parse_frontend_payload(&json, &root)?
        }
        other => {
            // Minimal inventory-only analysis for unsupported frontends.
            let mut g = ProjectGraph::new(
                root.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("project"),
            );
            g.files = files.clone();
            g.build_system = Some(match other {
                SourceLanguage::Python => "pip".into(),
                SourceLanguage::Rust => "cargo".into(),
                SourceLanguage::Go => "go".into(),
                _ => "unknown".into(),
            });
            let puir = PuirProgram {
                version: PUIR_SCHEMA_VERSION,
                modules: IndexMap::new(),
                metadata: IndexMap::new(),
            };
            (g, puir, None, None)
        }
    };

    // Merge inventory files if frontend omitted some.
    if graph.files.is_empty() {
        graph.files = files;
    }

    let types = infer_types(&puir);
    puir.metadata
        .insert("type_inference_count".into(), serde_json::json!(types.reports.len()));

    Ok(ProjectAnalysis {
        root: root.display().to_string(),
        primary_language: primary,
        language_mix,
        graph,
        puir,
        types,
        framework,
        database,
        analyzed_at: Utc::now(),
    })
}

fn inventory_files(root: &Path) -> Result<Vec<ProjectFile>, ParallaxError> {
    let mut out = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !matches!(
                name.as_ref(),
                "node_modules" | "target" | ".git" | ".parallax" | "dist" | "build" | "coverage"
            )
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let rel = entry
            .path()
            .strip_prefix(root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .replace('\\', "/");
        let ext = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let (role, language) = match ext.as_str() {
            "ts" | "tsx" => {
                let role = if rel.contains("test") || rel.contains("spec") {
                    "test"
                } else {
                    "source"
                };
                (role, Some("typescript".into()))
            }
            "js" | "jsx" | "mjs" => {
                let role = if rel.contains("test") || rel.contains("spec") {
                    "test"
                } else {
                    "source"
                };
                (role, Some("javascript".into()))
            }
            "py" => (
                if rel.contains("test") { "test" } else { "source" },
                Some("python".into()),
            ),
            "json" | "toml" | "yaml" | "yml" | "env" => ("config", None),
            "md" | "txt" => ("resource", None),
            _ => continue,
        };
        let bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
        out.push(ProjectFile {
            path: rel,
            role: role.into(),
            language,
            bytes,
        });
    }
    Ok(out)
}

#[derive(serde::Deserialize)]
struct FrontendPayload {
    graph: ProjectGraph,
    modules: Vec<Module>,
    framework: Option<String>,
    database: Option<String>,
}

fn parse_frontend_payload(
    json: &str,
    root: &Path,
) -> Result<(ProjectGraph, PuirProgram, Option<String>, Option<String>), ParallaxError> {
    let payload: FrontendPayload = serde_json::from_str(json).map_err(|e| {
        ParallaxError::new(
            ErrorCode::SerializationFailure,
            format!("invalid TypeScript frontend payload: {e}"),
        )
        .with_source("parallax-transmute")
        .with_operation("parse_frontend_payload")
        .with_diagnostic(json.chars().take(500).collect::<String>())
    })?;
    let mut puir = PuirProgram::new();
    for m in payload.modules {
        puir.modules.insert(m.id.clone(), m);
    }
    let mut graph = payload.graph;
    if graph.name.is_empty() {
        graph.name = root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("project")
            .into();
    }
    // Ensure entrypoints from package.json main
    if graph.entrypoints.is_empty() {
        let pkg = root.join("package.json");
        if let Ok(text) = fs::read_to_string(pkg) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(main) = v.get("main").and_then(|m| m.as_str()) {
                    graph.entrypoints.push(Entrypoint {
                        path: main.into(),
                        kind: "bin".into(),
                    });
                }
            }
        }
    }
    // Wire simple import edges
    for module in puir.modules.values() {
        let mid = format!("module:{}", module.id);
        graph.nodes.entry(mid.clone()).or_insert(GraphNode {
            id: mid.clone(),
            kind: GraphNodeKind::Module,
            name: module.id.clone(),
            file: Some(module.path.clone()),
            attrs: IndexMap::new(),
        });
        for imp in &module.imports {
            let to = format!("import:{}", imp.from);
            graph.nodes.entry(to.clone()).or_insert(GraphNode {
                id: to.clone(),
                kind: if imp.from.starts_with('.') {
                    GraphNodeKind::Module
                } else {
                    GraphNodeKind::Package
                },
                name: imp.from.clone(),
                file: None,
                attrs: IndexMap::new(),
            });
            graph.edges.push(GraphEdge {
                from: mid.clone(),
                to,
                kind: GraphEdgeKind::Imports,
            });
        }
    }
    Ok((graph, puir, payload.framework, payload.database))
}
