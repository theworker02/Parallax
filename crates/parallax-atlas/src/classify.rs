//! Project type classification.

use parallax_adapter_sdk::ProjectContext;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectKind {
    Cli,
    RestApi,
    GraphqlApi,
    Library,
    DesktopApp,
    WebApp,
    Worker,
    BackgroundService,
    MobileApp,
    DataPipeline,
    Monorepo,
    Unknown,
}

impl ProjectKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::RestApi => "rest_api",
            Self::GraphqlApi => "graphql_api",
            Self::Library => "library",
            Self::DesktopApp => "desktop_application",
            Self::WebApp => "web_application",
            Self::Worker => "worker",
            Self::BackgroundService => "background_service",
            Self::MobileApp => "mobile_app",
            Self::DataPipeline => "data_pipeline",
            Self::Monorepo => "monorepo",
            Self::Unknown => "unknown",
        }
    }
}

pub fn classify_project(ctx: &ProjectContext, framework_ids: &[String]) -> ProjectKind {
    if ctx
        .files
        .iter()
        .any(|f| f.starts_with("apps/") || f.starts_with("packages/") || f.starts_with("services/"))
        && (ctx.has_manifest("package.json")
            || ctx.has_manifest("Cargo.toml")
            || ctx.has_manifest("pnpm-workspace.yaml"))
    {
        return ProjectKind::Monorepo;
    }
    let fw = framework_ids.join(" ");
    if fw.contains("nextjs") || fw.contains("react") || fw.contains("vue") || fw.contains("svelte")
    {
        return ProjectKind::WebApp;
    }
    if fw.contains("express")
        || fw.contains("fastify")
        || fw.contains("nestjs")
        || fw.contains("fastapi")
        || fw.contains("flask")
        || fw.contains("django")
        || fw.contains("axum")
        || fw.contains("gin")
        || fw.contains("spring")
        || fw.contains("aspnet")
        || fw.contains("rails")
        || fw.contains("laravel")
    {
        return ProjectKind::RestApi;
    }
    if ctx.package_contains("graphql") {
        return ProjectKind::GraphqlApi;
    }
    if ctx.package_contains("clap")
        || ctx.package_contains("commander")
        || ctx.package_contains("typer")
        || ctx.package_contains("cobra")
    {
        return ProjectKind::Cli;
    }
    if ctx.package_contains("flutter") || ctx.has_manifest("pubspec.yaml") {
        return ProjectKind::MobileApp;
    }
    if ctx.has_manifest("Cargo.toml") && !fw.contains("axum") {
        // rust lib vs bin heuristic
        if ctx.files.iter().any(|f| f == "src/lib.rs")
            && !ctx.files.iter().any(|f| f == "src/main.rs")
        {
            return ProjectKind::Library;
        }
    }
    ProjectKind::Unknown
}
