//! Parallax Atlas — modular adapter orchestration.

#![deny(unsafe_code)]
#![allow(missing_docs)]

mod builtins;
mod classify;
mod compatibility;
mod context;
mod lockfile;
mod registry;
mod stack;

pub use classify::{classify_project, ProjectKind};
pub use compatibility::{pair_compatibility, CompatibilityReport, FeatureScore};
pub use context::build_project_context;
pub use lockfile::{AdapterLockfile, LockEntry};
pub use registry::{AdapterEntry, AdapterRegistry, RegisteredDetection};
pub use stack::{
    analyze_stack, AdapterStackPlan, CompletenessEstimate, StackAnalysis, StackComponent,
    TargetStackSuggestion,
};

use parallax_adapter_sdk::ADAPTER_SDK_VERSION;

/// Atlas orchestration format version.
pub const ATLAS_FORMAT_VERSION: u32 = 1;

/// Create a registry preloaded with built-in adapters.
pub fn builtin_registry() -> AdapterRegistry {
    let mut reg = AdapterRegistry::new();
    builtins::register_all(&mut reg);
    reg
}

/// SDK + Atlas version snapshot.
pub fn versions() -> serde_json::Value {
    serde_json::json!({
        "atlas_format": ATLAS_FORMAT_VERSION,
        "adapter_sdk": ADAPTER_SDK_VERSION,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn builtin_registry_has_core_adapters() {
        let reg = builtin_registry();
        assert!(reg.len() > 115);
        assert!(reg.get("typescript").is_some() || reg.get("parallax.typescript.source").is_some());
        assert!(reg.get("parallax.rust.target").is_some());
        assert!(reg.get("express").is_some() || reg.get("parallax.framework.express").is_some());
        assert!(reg.get("parallax.framework.hono").is_some());
        assert!(reg.get("parallax.validation.zod").is_some());
        assert!(reg.get("parallax.desktop.tauri").is_some());
        assert!(reg.get("parallax.codegen.openapi").is_some());
        assert!(reg.get("parallax.formatter.prettier").is_some());
        assert!(reg.get("parallax.linter.eslint").is_some());
    }

    #[test]
    fn analyze_detects_express_package_json() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"name":"demo","dependencies":{"express":"^4.18.0"},"devDependencies":{"vitest":"^1.0.0"}}"#,
        )
        .unwrap();
        fs::write(dir.path().join("index.ts"), "import express from 'express';\n").unwrap();
        let reg = builtin_registry();
        let a = analyze_stack(dir.path(), &reg, Some("rust")).unwrap();
        assert!(a
            .detected
            .iter()
            .any(|d| d.id.contains("express")));
        assert!(a
            .stack
            .selected
            .iter()
            .any(|s| s.adapter_id.contains("typescript")));
        let sug = a.stack.target_suggestion.unwrap();
        assert_eq!(sug.language, "rust");
        assert_eq!(sug.framework.as_deref(), Some("axum"));
    }

    #[test]
    fn analyze_detects_hono_and_pnpm() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"name":"demo","dependencies":{"hono":"^4.0.0"},"devDependencies":{"zod":"^3.0.0"}}"#,
        )
        .unwrap();
        fs::write(dir.path().join("pnpm-lock.yaml"), "lockfileVersion: 9.0\n").unwrap();
        fs::write(dir.path().join("index.ts"), "import { Hono } from 'hono';\n").unwrap();
        let reg = builtin_registry();
        let a = analyze_stack(dir.path(), &reg, Some("rust")).unwrap();
        assert!(a.detected.iter().any(|d| d.id.contains("hono")));
        assert!(a.detected.iter().any(|d| d.id.contains("pnpm")));
        assert!(a.detected.iter().any(|d| d.id.contains("zod")));
    }

    #[test]
    fn analyze_detects_nest_prisma_stack_fixture() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/stacks/nest-prisma");
        if !root.exists() {
            return;
        }
        let reg = builtin_registry();
        let a = analyze_stack(&root, &reg, Some("rust")).unwrap();
        assert!(a.detected.iter().any(|d| d.id.contains("nestjs")));
        assert!(a.detected.iter().any(|d| d.id.contains("prisma")));
        assert!(a.detected.iter().any(|d| d.id.contains("typescript")));
    }

    #[test]
    fn analyze_detects_tauri_desktop_fixture() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/stacks/tauri-desktop");
        if !root.exists() {
            return;
        }
        let reg = builtin_registry();
        let a = analyze_stack(&root, &reg, None).unwrap();
        assert!(a.detected.iter().any(|d| d.id.contains("tauri")));
    }

    #[test]
    fn compatibility_typescript_rust_is_strong() {
        let r = pair_compatibility("typescript", "rust");
        assert!(r.overall_pct >= 70);
        assert_eq!(r.overall, "stable");
    }
}

