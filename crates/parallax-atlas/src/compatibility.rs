//! Language-pair compatibility scoring.

use parallax_connectors::{pair_maturity, PairMaturity};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeatureScore {
    pub feature: String,
    pub score_pct: u8,
    pub notes: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompatibilityReport {
    pub source: String,
    pub target: String,
    pub overall: String,
    pub overall_pct: u8,
    pub features: Vec<FeatureScore>,
}

/// Honest compatibility matrix based on pair tier + known hard gaps.
pub fn pair_compatibility(source: &str, target: &str) -> CompatibilityReport {
    let maturity = pair_maturity(source, target);
    let (overall, base) = match maturity {
        PairMaturity::Tier1 => ("stable", 92u8),
        PairMaturity::Tier2 => ("beta", 78u8),
        PairMaturity::Experimental => ("experimental", 55u8),
        PairMaturity::Scaffold => ("scaffold", 35u8),
        PairMaturity::Unsupported => ("unsupported", 15u8),
    };

    let mut features = vec![
        FeatureScore {
            feature: "Core syntax".into(),
            score_pct: (base + 2).min(98),
            notes: "functions, control flow, literals".into(),
        },
        FeatureScore {
            feature: "Classes / objects".into(),
            score_pct: adjust(base, source, target, "oop"),
            notes: "class → struct/impl or class".into(),
        },
        FeatureScore {
            feature: "Async".into(),
            score_pct: adjust(base, source, target, "async"),
            notes: "Promise/asyncio ↔ Future/goroutine".into(),
        },
        FeatureScore {
            feature: "Nullability".into(),
            score_pct: adjust(base, source, target, "null"),
            notes: "optional / Option / nullable".into(),
        },
        FeatureScore {
            feature: "Reflection".into(),
            score_pct: 30.min(base),
            notes: "usually unsupported or manual review".into(),
        },
        FeatureScore {
            feature: "Metaprogramming".into(),
            score_pct: 25.min(base),
            notes: "decorators/macros/attributes — limited".into(),
        },
    ];

    // Framework-ish extras for known pairs
    if matches!(
        (source, target),
        ("typescript" | "javascript", "rust") | ("python", "rust")
    ) {
        features.push(FeatureScore {
            feature: "HTTP frameworks".into(),
            score_pct: if matches!(maturity, PairMaturity::Tier1) {
                90
            } else {
                70
            },
            notes: "Express/FastAPI → Axum pack".into(),
        });
    }

    let overall_pct = if features.is_empty() {
        base
    } else {
        (features.iter().map(|f| f.score_pct as u32).sum::<u32>() / features.len() as u32) as u8
    };

    CompatibilityReport {
        source: source.into(),
        target: target.into(),
        overall: overall.into(),
        overall_pct,
        features,
    }
}

fn adjust(base: u8, source: &str, target: &str, kind: &str) -> u8 {
    match (kind, source, target) {
        ("async", "typescript" | "javascript", "rust") => (base + 5).min(95),
        ("async", "python", "rust") => (base).saturating_sub(5),
        ("null", _, "rust") => (base).saturating_sub(8),
        ("oop", "java" | "csharp" | "kotlin", "rust") => base.saturating_sub(15),
        ("oop", "typescript", "go") => base.saturating_sub(10),
        _ => base.saturating_sub(5),
    }
}
