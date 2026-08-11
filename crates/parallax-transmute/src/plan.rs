//! Migration planner — ecosystem mapping before codegen.

use crate::deps::{ChosenDependency, DependencyMapDb};
use crate::options::{TargetStyle, TransmuteOptions};
use crate::packs::{pack_dep_overrides, MigrationPack};
use indexmap::IndexMap;
use parallax_project::{ProjectAnalysis, TargetLanguage};
use serde::{Deserialize, Serialize};

/// Machine-readable migration plan.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MigrationPlan {
    /// Pack used.
    pub pack_id: String,
    /// Source language.
    pub source_language: String,
    /// Target language.
    pub target_language: String,
    /// Target style.
    pub target_style: TargetStyle,
    /// Framework mappings.
    pub framework_mappings: IndexMap<String, String>,
    /// Dependency choices.
    pub dependencies: Vec<ChosenDependency>,
    /// Modules to translate (source relative paths).
    pub modules_to_translate: Vec<String>,
    /// Files to preserve as-is.
    pub files_to_preserve: Vec<String>,
    /// Infrastructure files to generate.
    pub infrastructure: Vec<String>,
    /// Stdlib API remaps (source API → target API).
    pub stdlib_mappings: IndexMap<String, String>,
    /// Notes.
    pub notes: Vec<String>,
}

/// Planner.
pub struct MigrationPlanner {
    deps: DependencyMapDb,
    pack: MigrationPack,
}

impl MigrationPlanner {
    /// Create.
    pub fn new(deps: DependencyMapDb, pack: MigrationPack) -> Self {
        Self { deps, pack }
    }

    /// Build a plan.
    pub fn plan(
        &self,
        analysis: &ProjectAnalysis,
        target: &TargetLanguage,
        opts: &TransmuteOptions,
    ) -> Result<MigrationPlan, parallax_core::ParallaxError> {
        let min_conf = if opts.strict { 0.85 } else { 0.55 };
        let mut chosen = Vec::new();
        for dep in &analysis.graph.packages {
            if dep.dev && matches!(dep.name.as_str(), "typescript" | "@types/node" | "@types/express" | "ts-node")
            {
                continue;
            }
            chosen.push(self.deps.resolve(&dep.ecosystem, &dep.name, min_conf));
        }
        // Annotate chosen deps with pack framework mapping notes (do not invent packages).
        for p in pack_dep_overrides(&self.pack) {
            if let Some(c) = chosen.iter_mut().find(|c| c.source == p.source) {
                if c.target.is_none() {
                    c.target = p.target.clone();
                    c.confidence = p.confidence;
                    c.notes = p.notes.clone();
                    c.manual_review = p.manual_review;
                }
            }
        }

        let mut modules = Vec::new();
        let mut preserve = Vec::new();
        for f in &analysis.graph.files {
            if opts.keep.iter().any(|k| {
                f.language.as_deref() == Some(k) || f.path.ends_with(k) || f.role == *k
            }) {
                preserve.push(f.path.clone());
                continue;
            }
            match f.role.as_str() {
                "source" | "test" => modules.push(f.path.clone()),
                "config" if f.path.ends_with(".env") => {
                    // never copy secrets
                }
                "config" | "resource" => preserve.push(f.path.clone()),
                _ => {}
            }
        }

        let mut stdlib = IndexMap::new();
        stdlib.insert("JSON.parse".into(), "serde_json::from_str".into());
        stdlib.insert("JSON.stringify".into(), "serde_json::to_string".into());
        stdlib.insert("process.env".into(), "std::env::var".into());
        stdlib.insert("pathlib.Path".into(), "std::path::PathBuf".into());
        stdlib.insert("os.getenv".into(), "std::env::var".into());
        stdlib.insert("fs.readFileSync".into(), "std::fs::read_to_string".into());

        let infrastructure = match target {
            TargetLanguage::Rust => vec![
                "Cargo.toml".into(),
                "src/main.rs".into(),
                "src/lib.rs".into(),
                ".env.example".into(),
                "PARALLAX_MIGRATION.md".into(),
            ],
            _ => vec!["PARALLAX_MIGRATION.md".into()],
        };

        Ok(MigrationPlan {
            pack_id: self.pack.id.clone(),
            source_language: analysis.primary_language.to_string(),
            target_language: target.to_string(),
            target_style: opts.target_style.clone(),
            framework_mappings: self.pack.framework_mappings.clone(),
            dependencies: chosen,
            modules_to_translate: modules,
            files_to_preserve: preserve,
            infrastructure,
            stdlib_mappings: stdlib,
            notes: self.pack.structure_notes.clone(),
        })
    }
}
