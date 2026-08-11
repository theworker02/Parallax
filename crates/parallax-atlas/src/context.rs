//! Build ProjectContext from a filesystem root.

use parallax_adapter_sdk::ProjectContext;
use parallax_core::{ErrorCode, ParallaxError};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

const SKIP: &[&str] = &[
    "node_modules",
    "target",
    ".git",
    "dist",
    "build",
    ".parallax",
    ".parallax-link",
    "vendor",
    "__pycache__",
    ".venv",
    "venv",
];

/// Scan a project directory into an adapter ProjectContext.
pub fn build_project_context(root: &Path) -> Result<ProjectContext, ParallaxError> {
    let root = root.canonicalize().map_err(|e| {
        ParallaxError::new(ErrorCode::Io, e.to_string()).with_source("parallax-atlas")
    })?;
    let mut ctx = ProjectContext::new(root.clone());
    let mut lang_counts: HashMap<String, usize> = HashMap::new();

    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !SKIP.iter().any(|s| *s == name)
        })
        .flatten()
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(&root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .replace('\\', "/");
        ctx.files.push(rel.clone());

        if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
            if let Some(lang) = ext_lang(ext) {
                *lang_counts.entry(lang.into()).or_default() += 1;
            }
        }

        let fname = entry.file_name().to_string_lossy();
        match fname.as_ref() {
            "package.json" | "Cargo.toml" | "pyproject.toml" | "requirements.txt"
            | "go.mod" | "pom.xml" | "build.gradle" | "build.gradle.kts" | "Gemfile"
            | "composer.json" | "Package.swift" | "pubspec.yaml" | "tsconfig.json"
            | "Dockerfile" | "docker-compose.yml" | "docker-compose.yaml" | "compose.yaml"
            | "vercel.json" | "render.yaml" | "Pipfile" | "pnpm-lock.yaml" | "yarn.lock"
            | "bunfig.toml" | "uv.lock" | "poetry.lock" | "CMakeLists.txt" | "meson.build"
            | "fly.toml" | "netlify.toml" | "railway.toml" | "railway.json" | "mix.exs"
            | "serverless.yml" | "serverless.yaml" | "template.yaml" | "samconfig.toml"
            | ".gitlab-ci.yml" | "rustfmt.toml" | "biome.json" | "biome.jsonc"
            | "eslint.config.js" | "eslint.config.mjs" | ".eslintrc.json" | ".eslintrc.js"
            | ".eslintrc.cjs" | "prettier.config.js" | "prettier.config.mjs" | ".prettierrc.json"
            | "wails.json" | "openapitools.json" | "codegen.yml" | "codegen.ts"
            | "electron-builder.yml" | "analysis_options.yaml" | "mypy.ini"
            | ".golangci.yml" | ".golangci.yaml" | ".rubocop.yml" | ".rubocop.yaml"
            | "project.clj" => {
                if let Ok(text) = fs::read_to_string(entry.path()) {
                    ctx.manifests.insert(fname.to_string(), text.clone());
                    extract_packages(&fname, &text, &mut ctx.packages);
                } else {
                    ctx.manifests.insert(fname.to_string(), String::new());
                }
            }
            _ => {}
        }
        if fname.ends_with(".csproj") {
            ctx.manifests
                .insert(fname.to_string(), String::new());
        }
        if fname == "tauri.conf.json" {
            if let Ok(text) = fs::read_to_string(entry.path()) {
                ctx.manifests.insert("tauri.conf.json".into(), text);
                ctx.packages.push("tauri".into());
            }
        }
        if fname.ends_with(".proto") {
            ctx.packages.push("protobuf".into());
        }
        if fname.ends_with("openapi.yaml")
            || fname.ends_with("openapi.yml")
            || fname.ends_with("openapi.json")
            || fname.ends_with("swagger.yaml")
            || fname.ends_with("swagger.json")
        {
            ctx.packages.push("openapi".into());
        }
        if fname == "build.sbt" {
            ctx.manifests.insert("build.sbt".into(), String::new());
        }
    }

    let total: usize = lang_counts.values().sum();
    if total > 0 {
        for (k, v) in lang_counts {
            ctx.language_mix
                .insert(k, (v as f64) * 100.0 / total as f64);
        }
    }
    Ok(ctx)
}

fn ext_lang(ext: &str) -> Option<&'static str> {
    match ext.to_ascii_lowercase().as_str() {
        "ts" | "tsx" => Some("typescript"),
        "js" | "jsx" | "mjs" | "cjs" => Some("javascript"),
        "py" => Some("python"),
        "rs" => Some("rust"),
        "go" => Some("go"),
        "java" => Some("java"),
        "kt" | "kts" => Some("kotlin"),
        "cs" => Some("csharp"),
        "rb" => Some("ruby"),
        "php" => Some("php"),
        "swift" => Some("swift"),
        "dart" => Some("dart"),
        "c" | "h" => Some("c"),
        "cpp" | "cc" | "cxx" | "hpp" => Some("cpp"),
        "lua" => Some("lua"),
        _ => None,
    }
}

fn extract_packages(manifest: &str, text: &str, out: &mut Vec<String>) {
    match manifest {
        "package.json" => {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(text) {
                for key in ["dependencies", "devDependencies", "peerDependencies"] {
                    if let Some(obj) = v.get(key).and_then(|x| x.as_object()) {
                        out.extend(obj.keys().cloned());
                    }
                }
            }
        }
        "Cargo.toml" => {
            for line in text.lines() {
                let t = line.trim();
                if t.starts_with('[') || t.is_empty() || t.starts_with('#') {
                    continue;
                }
                if let Some((name, _)) = t.split_once('=') {
                    let name = name.trim();
                    if !name.is_empty() && !name.contains('.') {
                        out.push(name.to_string());
                    }
                }
            }
        }
        "requirements.txt" | "Pipfile" => {
            for line in text.lines() {
                let t = line.trim();
                if t.is_empty() || t.starts_with('#') || t.starts_with('[') {
                    continue;
                }
                let name = t
                    .split(&['=', '>', '<', '!', '~', ';', '['][..])
                    .next()
                    .unwrap_or("")
                    .trim();
                if !name.is_empty() {
                    out.push(name.to_string());
                }
            }
        }
        "pyproject.toml" => {
            for needle in [
                "fastapi", "flask", "django", "sqlalchemy", "pytest", "httpx", "pydantic",
                "litestar", "sanic", "click", "typer", "uvicorn",
            ] {
                if text.to_ascii_lowercase().contains(needle) {
                    out.push(needle.into());
                }
            }
        }
        "mix.exs" => {
            for needle in ["phoenix", "ecto"] {
                if text.to_ascii_lowercase().contains(needle) {
                    out.push(needle.into());
                }
            }
        }
        "Gemfile" => {
            for line in text.lines() {
                if let Some(rest) = line.trim().strip_prefix("gem ") {
                    let name = rest
                        .trim()
                        .trim_matches(|c| c == '\'' || c == '"' || c == ',')
                        .split(',')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_matches(|c| c == '\'' || c == '"');
                    if !name.is_empty() {
                        out.push(name.to_string());
                    }
                }
            }
        }
        "go.mod" => {
            for line in text.lines() {
                let t = line.trim();
                if t.starts_with("require ") || (!t.is_empty() && !t.starts_with("module ") && !t.starts_with("go ") && !t.starts_with("//") && !t.starts_with(")")) {
                    let parts: Vec<_> = t.split_whitespace().collect();
                    if let Some(p) = parts.first() {
                        if p.contains('.') || p.contains('/') {
                            out.push((*p).to_string());
                        }
                    }
                }
            }
        }
        "composer.json" => {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(text) {
                for key in ["require", "require-dev"] {
                    if let Some(obj) = v.get(key).and_then(|x| x.as_object()) {
                        out.extend(obj.keys().cloned());
                    }
                }
            }
        }
        _ => {}
    }
}
