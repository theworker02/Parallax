//! Stack analysis — detect + select adapter stack + target suggestions.

use crate::classify::{classify_project, ProjectKind};
use crate::context::build_project_context;
use crate::registry::{AdapterRegistry, RegisteredDetection};
use parallax_connectors::{pair_maturity, PairMaturity};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StackComponent {
    pub role: String,
    pub adapter_id: String,
    pub name: String,
    pub maturity: String,
    pub confidence: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdapterStackPlan {
    pub selected: Vec<StackComponent>,
    pub conflicts_resolved: Vec<String>,
    pub target_suggestion: Option<TargetStackSuggestion>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TargetStackSuggestion {
    pub language: String,
    pub framework: Option<String>,
    pub orm: Option<String>,
    pub async_runtime: Option<String>,
    pub rationale: Vec<String>,
    pub pair_maturity: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StackAnalysis {
    pub root: String,
    pub project_kind: String,
    pub language_mix: std::collections::HashMap<String, f64>,
    pub detected: Vec<RegisteredDetection>,
    pub resolved: Vec<RegisteredDetection>,
    pub stack: AdapterStackPlan,
    pub completeness_estimate: CompletenessEstimate,
    pub unsupported: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompletenessEstimate {
    pub exact_pct: u8,
    pub high_confidence_pct: u8,
    pub review_pct: u8,
    pub unsupported_pct: u8,
    pub notes: String,
}

/// Analyze a project stack using the Atlas registry.
pub fn analyze_stack(
    root: &Path,
    registry: &AdapterRegistry,
    to: Option<&str>,
) -> Result<StackAnalysis, parallax_core::ParallaxError> {
    let mut ctx = build_project_context(root)?;
    if let Some(t) = to {
        ctx.hints.insert("to".into(), t.to_string());
    }

    let detected = registry.detect_all(&ctx);
    let resolved = registry.resolve_conflicts(&detected);

    let mut conflicts = Vec::new();
    // Report when multiple of same type were detected before resolve
    let mut by_type: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for d in &detected {
        by_type
            .entry(d.adapter_type.clone())
            .or_default()
            .push(d.id.clone());
    }
    for (ty, ids) in &by_type {
        if ids.len() > 1 {
            let winner = resolved
                .iter()
                .find(|r| r.adapter_type == *ty)
                .map(|r| r.id.as_str())
                .unwrap_or("?");
            conflicts.push(format!("{ty}: {} → selected {winner}", ids.join(", ")));
        }
    }

    let fw_ids: Vec<String> = resolved
        .iter()
        .filter(|d| d.adapter_type == "framework")
        .map(|d| d.id.clone())
        .collect();
    let kind = classify_project(&ctx, &fw_ids);

    let selected: Vec<StackComponent> = resolved
        .iter()
        .map(|d| StackComponent {
            role: d.adapter_type.clone(),
            adapter_id: d.id.clone(),
            name: d.name.clone(),
            maturity: d.maturity.clone(),
            confidence: d.detection.confidence.as_str().to_string(),
        })
        .collect();

    let primary_lang = primary_language(&ctx, &resolved);
    let target_suggestion = to.map(|t| suggest_target(&primary_lang, t, &resolved, kind));

    let completeness = estimate_completeness(&primary_lang, &resolved, to);
    let unsupported = collect_unsupported(&resolved);

    Ok(StackAnalysis {
        root: ctx.root.display().to_string(),
        project_kind: kind.as_str().to_string(),
        language_mix: ctx.language_mix,
        detected,
        resolved,
        stack: AdapterStackPlan {
            selected,
            conflicts_resolved: conflicts,
            target_suggestion,
        },
        completeness_estimate: completeness,
        unsupported,
    })
}

fn primary_language(
    ctx: &parallax_adapter_sdk::ProjectContext,
    resolved: &[RegisteredDetection],
) -> String {
    if let Some(src) = resolved
        .iter()
        .find(|d| d.adapter_type == "source-language")
    {
        if src.id.contains("typescript") {
            return "typescript".into();
        }
        if src.id.contains("javascript") {
            return "javascript".into();
        }
        if src.id.contains("python") {
            return "python".into();
        }
        if src.id.contains("go") {
            return "go".into();
        }
        if src.id.contains("java") {
            return "java".into();
        }
        if src.id.contains("ruby") {
            return "ruby".into();
        }
        if src.id.contains("csharp") {
            return "csharp".into();
        }
    }
    ctx.language_mix
        .iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(k, _)| k.clone())
        .unwrap_or_else(|| "unknown".into())
}

fn suggest_target(
    source: &str,
    target: &str,
    resolved: &[RegisteredDetection],
    kind: ProjectKind,
) -> TargetStackSuggestion {
    let mut rationale = Vec::new();
    let maturity = pair_maturity(source, target);
    let framework = match (source, target, kind) {
        (_, "rust", ProjectKind::RestApi) => {
            rationale.push("REST API → Axum is the Tier-1 Parallax mapping".into());
            Some("axum".into())
        }
        (_, "rust", _) => {
            rationale
                .push("Default Rust web stack when HTTP detected; otherwise library style".into());
            if resolved
                .iter()
                .any(|d| d.id.contains("express") || d.id.contains("fastapi"))
            {
                Some("axum".into())
            } else {
                None
            }
        }
        (_, "go", ProjectKind::RestApi) => {
            rationale.push(
                "Go REST APIs prefer Chi/Gin; Chi selected for stdlib-friendly routing".into(),
            );
            Some("chi".into())
        }
        (_, "python", ProjectKind::RestApi) => Some("fastapi".into()),
        _ => None,
    };
    let orm = if resolved
        .iter()
        .any(|d| d.id.contains("prisma") || d.id.contains("sqlalchemy"))
    {
        match target {
            "rust" => {
                rationale.push(
                    "ORM present → SQLx (query-first) over Diesel for migration safety".into(),
                );
                Some("sqlx".into())
            }
            "go" => Some("database/sql".into()),
            _ => None,
        }
    } else {
        None
    };
    let async_runtime = match target {
        "rust" => Some("tokio".into()),
        "python" => Some("asyncio".into()),
        "go" => Some("goroutines".into()),
        _ => None,
    };
    TargetStackSuggestion {
        language: target.into(),
        framework,
        orm,
        async_runtime,
        rationale,
        pair_maturity: match maturity {
            PairMaturity::Tier1 => "tier1",
            PairMaturity::Tier2 => "tier2",
            PairMaturity::Experimental => "experimental",
            PairMaturity::Scaffold => "scaffold",
            PairMaturity::Unsupported => "unsupported",
        }
        .into(),
    }
}

fn estimate_completeness(
    primary_lang: &str,
    resolved: &[RegisteredDetection],
    to: Option<&str>,
) -> CompletenessEstimate {
    let has_src = resolved.iter().any(|d| d.adapter_type == "source-language");
    let has_fw = resolved.iter().any(|d| d.adapter_type == "framework");
    let has_test = resolved.iter().any(|d| d.adapter_type == "test-framework");
    let has_build = resolved.iter().any(|d| d.adapter_type == "build-system");
    let stable_src = resolved.iter().any(|d| {
        d.adapter_type == "source-language" && (d.maturity == "stable" || d.maturity == "beta")
    });
    let target = to.unwrap_or("");
    let pair_ok = matches!(
        pair_maturity(primary_lang, target),
        PairMaturity::Tier1 | PairMaturity::Tier2
    ) || target.is_empty();

    // Heuristic based on actual detections — not fabricated precision.
    let mut exact = 40u8;
    if has_src {
        exact += 15;
    }
    if stable_src {
        exact += 10;
    }
    if has_fw {
        exact += 10;
    }
    if has_test {
        exact += 5;
    }
    if has_build {
        exact += 5;
    }
    if pair_ok && !target.is_empty() {
        exact += 10;
    }
    exact = exact.min(85);
    let high = ((100 - exact) as f32 * 0.45) as u8;
    let review = ((100 - exact - high) as f32 * 0.7) as u8;
    let unsupported = 100u8.saturating_sub(exact + high + review);
    CompletenessEstimate {
        exact_pct: exact,
        high_confidence_pct: high,
        review_pct: review,
        unsupported_pct: unsupported,
        notes: "Estimates derived from detected adapter maturity and pair tier — not formal proof"
            .into(),
    }
}

fn collect_unsupported(resolved: &[RegisteredDetection]) -> Vec<String> {
    let mut out = Vec::new();
    for d in resolved {
        if d.maturity == "scaffold" || d.maturity == "parse_only" {
            out.push(format!(
                "{} ({}) — detection only; translation not implemented",
                d.name, d.id
            ));
        }
    }
    out
}
