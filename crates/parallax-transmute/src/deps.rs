//! Dependency mapping knowledge layer.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// One candidate equivalent package.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DepEquivalent {
    /// Target ecosystem (crates.io, go, pypi, npm).
    pub ecosystem: String,
    /// Package name.
    pub name: String,
    /// Confidence 0.0–1.0.
    pub confidence: f64,
    /// API similarity 0.0–1.0.
    pub api_similarity: f64,
    /// Feature overlap 0.0–1.0.
    pub feature_overlap: f64,
    /// Async model note.
    pub async_model: String,
    /// Maturity note.
    pub maturity: String,
    /// Migration notes.
    pub notes: String,
}

/// Mapping from a source package.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DepMapping {
    /// Source ecosystem.
    pub ecosystem: String,
    /// Source package.
    pub name: String,
    /// Equivalents ordered by preference.
    pub equivalents: Vec<DepEquivalent>,
}

/// Chosen mapping for the plan.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChosenDependency {
    /// Source package.
    pub source: String,
    /// Chosen target package (None if unresolved).
    pub target: Option<String>,
    /// Confidence of choice.
    pub confidence: f64,
    /// Alternatives not chosen.
    pub alternatives: Vec<String>,
    /// Notes / why manual review.
    pub notes: String,
    /// Requires manual review.
    pub manual_review: bool,
}

/// Database of mappings.
#[derive(Clone, Debug, Default)]
pub struct DependencyMapDb {
    /// Keyed by "ecosystem:name".
    pub entries: IndexMap<String, DepMapping>,
}

impl DependencyMapDb {
    /// Built-in knowledge for the first migration packs.
    pub fn builtin() -> Self {
        let mut db = Self::default();
        db.insert(DepMapping {
            ecosystem: "npm".into(),
            name: "express".into(),
            equivalents: vec![
                eq(
                    "crates.io",
                    "axum",
                    0.9,
                    0.85,
                    0.8,
                    "async",
                    "mature",
                    "HTTP server framework",
                ),
                eq(
                    "crates.io",
                    "actix-web",
                    0.85,
                    0.8,
                    0.85,
                    "async",
                    "mature",
                    "Alternative web framework",
                ),
            ],
        });
        db.insert(DepMapping {
            ecosystem: "npm".into(),
            name: "axios".into(),
            equivalents: vec![eq(
                "crates.io",
                "reqwest",
                0.92,
                0.88,
                0.9,
                "async",
                "mature",
                "HTTP client",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "npm".into(),
            name: "zod".into(),
            equivalents: vec![eq(
                "crates.io",
                "serde",
                0.7,
                0.55,
                0.6,
                "n/a",
                "mature",
                "Use serde + validator for runtime checks",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "npm".into(),
            name: "dotenv".into(),
            equivalents: vec![eq(
                "crates.io",
                "dotenvy",
                0.95,
                0.95,
                0.95,
                "n/a",
                "mature",
                "Environment loading",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "npm".into(),
            name: "vitest".into(),
            equivalents: vec![eq(
                "crates.io",
                "cargo-test",
                0.9,
                0.7,
                0.75,
                "n/a",
                "built-in",
                "Use #[cfg(test)] / cargo test",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "npm".into(),
            name: "jest".into(),
            equivalents: vec![eq(
                "crates.io",
                "cargo-test",
                0.88,
                0.65,
                0.7,
                "n/a",
                "built-in",
                "Use #[cfg(test)] / cargo test",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "pypi".into(),
            name: "requests".into(),
            equivalents: vec![eq(
                "crates.io",
                "reqwest",
                0.9,
                0.85,
                0.85,
                "async/blocking",
                "mature",
                "HTTP client",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "pypi".into(),
            name: "fastapi".into(),
            equivalents: vec![
                eq(
                    "crates.io",
                    "axum",
                    0.85,
                    0.75,
                    0.8,
                    "async",
                    "mature",
                    "HTTP API framework",
                ),
                eq(
                    "go",
                    "chi",
                    0.7,
                    0.55,
                    0.65,
                    "goroutines",
                    "mature",
                    "Go HTTP router",
                ),
                eq(
                    "npm",
                    "fastify",
                    0.75,
                    0.7,
                    0.7,
                    "async",
                    "mature",
                    "Node HTTP framework",
                ),
            ],
        });
        db.insert(DepMapping {
            ecosystem: "pypi".into(),
            name: "flask".into(),
            equivalents: vec![eq(
                "crates.io",
                "axum",
                0.7,
                0.55,
                0.6,
                "async",
                "mature",
                "Lightweight HTTP → Axum",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "pypi".into(),
            name: "sqlalchemy".into(),
            equivalents: vec![
                eq(
                    "crates.io",
                    "sqlx",
                    0.8,
                    0.6,
                    0.7,
                    "async",
                    "mature",
                    "Query-first SQL",
                ),
                eq(
                    "crates.io",
                    "diesel",
                    0.75,
                    0.65,
                    0.75,
                    "sync",
                    "mature",
                    "ORM-style",
                ),
                eq(
                    "crates.io",
                    "sea-orm",
                    0.78,
                    0.7,
                    0.75,
                    "async",
                    "growing",
                    "Async ORM",
                ),
            ],
        });
        db.insert(DepMapping {
            ecosystem: "pypi".into(),
            name: "pytest".into(),
            equivalents: vec![eq(
                "crates.io",
                "cargo-test",
                0.88,
                0.65,
                0.7,
                "n/a",
                "built-in",
                "Use #[cfg(test)] / cargo test",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "pypi".into(),
            name: "httpx".into(),
            equivalents: vec![eq(
                "crates.io",
                "reqwest",
                0.9,
                0.85,
                0.88,
                "async",
                "mature",
                "HTTP client",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "npm".into(),
            name: "prisma".into(),
            equivalents: vec![
                eq(
                    "crates.io",
                    "sqlx",
                    0.75,
                    0.5,
                    0.65,
                    "async",
                    "mature",
                    "Prefer regenerating from schema",
                ),
                eq(
                    "crates.io",
                    "sea-orm",
                    0.72,
                    0.6,
                    0.7,
                    "async",
                    "growing",
                    "ORM closer to Prisma models",
                ),
            ],
        });
        db.insert(DepMapping {
            ecosystem: "npm".into(),
            name: "fastify".into(),
            equivalents: vec![eq(
                "crates.io",
                "axum",
                0.88,
                0.8,
                0.82,
                "async",
                "mature",
                "HTTP server framework",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "npm".into(),
            name: "@nestjs/core".into(),
            equivalents: vec![eq(
                "crates.io",
                "axum",
                0.7,
                0.45,
                0.55,
                "async",
                "mature",
                "NestJS DI/modules need manual structure mapping",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "maven".into(),
            name: "spring-boot".into(),
            equivalents: vec![
                eq(
                    "crates.io",
                    "axum",
                    0.55,
                    0.35,
                    0.45,
                    "async",
                    "mature",
                    "Experimental mapping",
                ),
                eq(
                    "go",
                    "gin",
                    0.5,
                    0.4,
                    0.45,
                    "goroutines",
                    "mature",
                    "Experimental",
                ),
            ],
        });
        db.insert(DepMapping {
            ecosystem: "nuget".into(),
            name: "aspnetcore".into(),
            equivalents: vec![eq(
                "crates.io",
                "axum",
                0.5,
                0.35,
                0.4,
                "async",
                "mature",
                "Experimental mapping",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "rubygems".into(),
            name: "rails".into(),
            equivalents: vec![eq(
                "crates.io",
                "axum",
                0.45,
                0.3,
                0.35,
                "async",
                "mature",
                "Rails → Axum is highly structural; experimental",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "go".into(),
            name: "gin-gonic/gin".into(),
            equivalents: vec![eq(
                "crates.io",
                "axum",
                0.8,
                0.7,
                0.75,
                "async",
                "mature",
                "HTTP router/framework",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "npm".into(),
            name: "hono".into(),
            equivalents: vec![eq(
                "crates.io",
                "axum",
                0.85,
                0.75,
                0.78,
                "async",
                "mature",
                "Lightweight HTTP router",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "npm".into(),
            name: "koa".into(),
            equivalents: vec![eq(
                "crates.io",
                "axum",
                0.8,
                0.65,
                0.7,
                "async",
                "mature",
                "Middleware-centric HTTP",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "npm".into(),
            name: "mocha".into(),
            equivalents: vec![eq(
                "crates.io",
                "cargo-test",
                0.85,
                0.6,
                0.65,
                "n/a",
                "built-in",
                "Use #[cfg(test)] / cargo test",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "npm".into(),
            name: "drizzle-orm".into(),
            equivalents: vec![
                eq(
                    "crates.io",
                    "sqlx",
                    0.78,
                    0.55,
                    0.65,
                    "async",
                    "mature",
                    "Schema-first SQL",
                ),
                eq(
                    "crates.io",
                    "sea-orm",
                    0.75,
                    0.6,
                    0.68,
                    "async",
                    "growing",
                    "ORM-style models",
                ),
            ],
        });
        db.insert(DepMapping {
            ecosystem: "pypi".into(),
            name: "litestar".into(),
            equivalents: vec![eq(
                "crates.io",
                "axum",
                0.82,
                0.7,
                0.75,
                "async",
                "growing",
                "ASGI API framework",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "pypi".into(),
            name: "sanic".into(),
            equivalents: vec![eq(
                "crates.io",
                "axum",
                0.75,
                0.6,
                0.65,
                "async",
                "mature",
                "Async HTTP framework",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "pypi".into(),
            name: "pydantic".into(),
            equivalents: vec![eq(
                "crates.io",
                "serde",
                0.72,
                0.6,
                0.65,
                "n/a",
                "mature",
                "Use serde + validator for runtime checks",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "go".into(),
            name: "gofiber/fiber".into(),
            equivalents: vec![
                eq(
                    "crates.io",
                    "axum",
                    0.78,
                    0.65,
                    0.7,
                    "async",
                    "mature",
                    "HTTP framework",
                ),
                eq(
                    "go",
                    "gin-gonic/gin",
                    0.85,
                    0.8,
                    0.82,
                    "goroutines",
                    "mature",
                    "Stay in Go",
                ),
            ],
        });
        db.insert(DepMapping {
            ecosystem: "go".into(),
            name: "labstack/echo".into(),
            equivalents: vec![
                eq(
                    "crates.io",
                    "axum",
                    0.76,
                    0.62,
                    0.68,
                    "async",
                    "mature",
                    "HTTP framework",
                ),
                eq(
                    "go",
                    "gin-gonic/gin",
                    0.82,
                    0.75,
                    0.78,
                    "goroutines",
                    "mature",
                    "Stay in Go",
                ),
            ],
        });
        db.insert(DepMapping {
            ecosystem: "go".into(),
            name: "gorm.io/gorm".into(),
            equivalents: vec![
                eq(
                    "crates.io",
                    "sqlx",
                    0.7,
                    0.5,
                    0.6,
                    "async",
                    "mature",
                    "Query-first SQL",
                ),
                eq(
                    "crates.io",
                    "sea-orm",
                    0.68,
                    0.55,
                    0.58,
                    "async",
                    "growing",
                    "ORM-style",
                ),
            ],
        });
        db.insert(DepMapping {
            ecosystem: "npm".into(),
            name: "commander".into(),
            equivalents: vec![eq(
                "crates.io",
                "clap",
                0.82,
                0.7,
                0.75,
                "n/a",
                "mature",
                "CLI argument parsing",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "go".into(),
            name: "spf13/cobra".into(),
            equivalents: vec![eq(
                "crates.io",
                "clap",
                0.85,
                0.72,
                0.78,
                "n/a",
                "mature",
                "CLI subcommands and flags",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "pypi".into(),
            name: "click".into(),
            equivalents: vec![eq(
                "crates.io",
                "clap",
                0.78,
                0.65,
                0.7,
                "n/a",
                "mature",
                "CLI decorator style → clap derive",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "pypi".into(),
            name: "typer".into(),
            equivalents: vec![eq(
                "crates.io",
                "clap",
                0.8,
                0.68,
                0.72,
                "n/a",
                "growing",
                "Type-hint CLI → clap derive",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "npm".into(),
            name: "prettier".into(),
            equivalents: vec![eq(
                "crates.io",
                "rustfmt",
                0.6,
                0.4,
                0.45,
                "n/a",
                "built-in",
                "Use rustfmt for Rust targets; no direct prettier equivalent",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "npm".into(),
            name: "eslint".into(),
            equivalents: vec![eq(
                "crates.io",
                "clippy",
                0.55,
                0.35,
                0.4,
                "n/a",
                "built-in",
                "Static analysis via clippy + deny lints in CI",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "pypi".into(),
            name: "black".into(),
            equivalents: vec![eq(
                "crates.io",
                "rustfmt",
                0.5,
                0.35,
                0.4,
                "n/a",
                "built-in",
                "Formatting is language-specific; use rustfmt on Rust output",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "pypi".into(),
            name: "ruff".into(),
            equivalents: vec![eq(
                "crates.io",
                "clippy",
                0.52,
                0.3,
                0.35,
                "n/a",
                "built-in",
                "Lint/format split: clippy + rustfmt on Rust",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "npm".into(),
            name: "@tauri-apps/api".into(),
            equivalents: vec![eq(
                "crates.io",
                "tauri",
                0.88,
                0.75,
                0.8,
                "async",
                "mature",
                "Stay on Tauri when targeting Rust desktop",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "npm".into(),
            name: "electron".into(),
            equivalents: vec![eq(
                "crates.io",
                "tauri",
                0.65,
                0.45,
                0.5,
                "async",
                "mature",
                "Electron → Tauri requires UI rewrite; experimental",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "go".into(),
            name: "google.golang.org/protobuf".into(),
            equivalents: vec![
                eq(
                    "crates.io",
                    "prost",
                    0.85,
                    0.7,
                    0.75,
                    "n/a",
                    "mature",
                    "Protobuf codegen",
                ),
                eq(
                    "crates.io",
                    "tonic",
                    0.8,
                    0.65,
                    0.7,
                    "async",
                    "mature",
                    "gRPC + prost",
                ),
            ],
        });
        db.insert(DepMapping {
            ecosystem: "npm".into(),
            name: "@nestjs/swagger".into(),
            equivalents: vec![eq(
                "crates.io",
                "utoipa",
                0.72,
                0.55,
                0.6,
                "async",
                "growing",
                "OpenAPI docs for Axum",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "maven".into(),
            name: "quarkus".into(),
            equivalents: vec![eq(
                "crates.io",
                "axum",
                0.5,
                0.3,
                0.35,
                "async",
                "mature",
                "Quarkus → Axum is structural; experimental",
            )],
        });
        db.insert(DepMapping {
            ecosystem: "composer".into(),
            name: "symfony/framework-bundle".into(),
            equivalents: vec![eq(
                "crates.io",
                "axum",
                0.45,
                0.28,
                0.32,
                "async",
                "mature",
                "Symfony → Axum highly structural; experimental",
            )],
        });
        db
    }

    fn insert(&mut self, m: DepMapping) {
        self.entries
            .insert(format!("{}:{}", m.ecosystem, m.name), m);
    }

    /// List known source package names (ecosystem:name).
    pub fn list_keys(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }

    /// Lookup by exact `ecosystem:name` or bare package name.
    pub fn lookup(&self, query: &str) -> Option<&DepMapping> {
        if let Some(m) = self.entries.get(query) {
            return Some(m);
        }
        let q = query.to_ascii_lowercase();
        self.entries.values().find(|m| {
            m.name.eq_ignore_ascii_case(&q)
                || format!("{}:{}", m.ecosystem, m.name).eq_ignore_ascii_case(&q)
        })
    }

    /// Resolve a source package into a chosen mapping.
    pub fn resolve(&self, ecosystem: &str, name: &str, min_confidence: f64) -> ChosenDependency {
        let key = format!("{ecosystem}:{name}");
        if let Some(m) = self.entries.get(&key) {
            let alts: Vec<String> = m.equivalents.iter().map(|e| e.name.clone()).collect();
            if let Some(best) = m.equivalents.first() {
                let manual = best.confidence < min_confidence;
                return ChosenDependency {
                    source: name.to_string(),
                    target: if manual {
                        None
                    } else {
                        Some(best.name.clone())
                    },
                    confidence: best.confidence,
                    alternatives: alts,
                    notes: best.notes.clone(),
                    manual_review: manual,
                };
            }
        }
        ChosenDependency {
            source: name.to_string(),
            target: None,
            confidence: 0.0,
            alternatives: Vec::new(),
            notes: "No known equivalent; manual review required".into(),
            manual_review: true,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn eq(
    eco: &str,
    name: &str,
    confidence: f64,
    api: f64,
    feat: f64,
    async_model: &str,
    maturity: &str,
    notes: &str,
) -> DepEquivalent {
    DepEquivalent {
        ecosystem: eco.into(),
        name: name.into(),
        confidence,
        api_similarity: api,
        feature_overlap: feat,
        async_model: async_model.into(),
        maturity: maturity.into(),
        notes: notes.into(),
    }
}
