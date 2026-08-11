//! Project observatory — semantic inspection without migrating.

#![deny(unsafe_code)]
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObservatoryReport {
    pub root: String,
    pub languages: HashMap<String, usize>,
    pub frameworks: Vec<String>,
    pub dynamic_signals: Vec<DynamicSignal>,
    pub effects: Vec<String>,
    pub protocols: Vec<String>,
    pub concurrency: Vec<String>,
    pub migration_barriers: Vec<String>,
    pub notes: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynamicSignal {
    pub kind: String,
    pub count: usize,
    pub samples: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct ProjectObserver;

impl ProjectObserver {
    pub fn observe(&self, root: &Path) -> Result<ObservatoryReport, String> {
        let root = root
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let mut languages = HashMap::new();
        let mut frameworks = Vec::new();
        let mut dynamic = HashMap::<String, Vec<String>>::new();
        let mut effects = Vec::new();
        let mut protocols = Vec::new();
        let mut concurrency = Vec::new();
        let mut barriers = Vec::new();
        let mut package_blob = String::new();

        let skip = [
            "node_modules", "target", ".git", "dist", "build", ".venv", "venv", "__pycache__",
        ];

        for entry in WalkDir::new(&root)
            .into_iter()
            .filter_entry(|e| {
                let n = e.file_name().to_string_lossy();
                !skip.iter().any(|s| *s == n)
            })
            .flatten()
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let rel = path
                .strip_prefix(&root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let lang = match ext {
                    "py" => "python",
                    "ts" | "tsx" => "typescript",
                    "js" | "jsx" => "javascript",
                    "rs" => "rust",
                    "go" => "go",
                    "rb" => "ruby",
                    "java" => "java",
                    _ => "",
                };
                if !lang.is_empty() {
                    *languages.entry(lang.into()).or_default() += 1;
                }
            }
            let name = entry.file_name().to_string_lossy();
            if matches!(
                name.as_ref(),
                "package.json" | "requirements.txt" | "pyproject.toml" | "Cargo.toml" | "go.mod"
            ) {
                if let Ok(t) = fs::read_to_string(path) {
                    package_blob.push_str(&t);
                    package_blob.push('\n');
                }
            }
            if ext_is(path, &["py", "js", "ts", "tsx", "rb", "java"]) {
                if let Ok(text) = fs::read_to_string(path) {
                    scan_source(&text, &rel, &mut dynamic, &mut effects, &mut concurrency, &mut barriers);
                }
            }
        }

        detect_frameworks(&package_blob, &mut frameworks);
        detect_protocols(&package_blob, &mut protocols);

        let dynamic_signals: Vec<_> = dynamic
            .into_iter()
            .map(|(kind, samples)| DynamicSignal {
                count: samples.len(),
                samples: samples.into_iter().take(5).collect(),
                kind,
            })
            .collect();

        Ok(ObservatoryReport {
            root: root.display().to_string(),
            languages,
            frameworks,
            dynamic_signals,
            effects,
            protocols,
            concurrency,
            migration_barriers: barriers,
            notes: "Observatory is static heuristics — not a full dynamic trace".into(),
        })
    }
}

fn ext_is(path: &Path, exts: &[&str]) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| exts.contains(&e))
}

fn scan_source(
    text: &str,
    rel: &str,
    dynamic: &mut HashMap<String, Vec<String>>,
    effects: &mut Vec<String>,
    concurrency: &mut Vec<String>,
    barriers: &mut Vec<String>,
) {
    let patterns = [
        ("getattr", "getattr"),
        ("setattr", "setattr"),
        ("eval(", "eval"),
        ("exec(", "exec"),
        ("__import__", "dynamic_import"),
        ("method_missing", "method_missing"),
        ("Proxy(", "proxy"),
        ("Class.forName", "reflection"),
        ("define_method", "metaprogram"),
        ("monkey", "monkey_patch"),
        ("ctypes", "c_extension"),
        ("ffi.", "ffi"),
    ];
    for (needle, kind) in patterns {
        if text.contains(needle) {
            dynamic
                .entry(kind.into())
                .or_default()
                .push(format!("{rel}: {needle}"));
            if matches!(kind, "eval" | "exec" | "c_extension" | "method_missing") {
                let b = format!("{kind} in {rel}");
                if !barriers.contains(&b) {
                    barriers.push(b);
                }
            }
        }
    }
    if text.contains("asyncio") || text.contains("async def") || text.contains("Promise") {
        push_unique(concurrency, "async/promises");
    }
    if text.contains("open(") || text.contains("fs.") {
        push_unique(effects, "filesystem");
    }
    if text.contains("requests.") || text.contains("fetch(") || text.contains("axios") {
        push_unique(effects, "network");
    }
}

fn detect_frameworks(blob: &str, out: &mut Vec<String>) {
    for (n, fw) in [
        ("express", "express"),
        ("fastapi", "fastapi"),
        ("flask", "flask"),
        ("django", "django"),
        ("sqlalchemy", "sqlalchemy"),
        ("prisma", "prisma"),
        ("axum", "axum"),
    ] {
        if blob.to_ascii_lowercase().contains(n) {
            push_unique(out, fw);
        }
    }
}

fn detect_protocols(blob: &str, out: &mut Vec<String>) {
    if blob.contains("express") || blob.contains("fastapi") || blob.contains("flask") {
        push_unique(out, "http");
    }
    if blob.contains("websocket") || blob.contains("ws") {
        push_unique(out, "websocket");
    }
}

fn push_unique(v: &mut Vec<String>, s: &str) {
    if !v.iter().any(|x| x == s) {
        v.push(s.into());
    }
}
