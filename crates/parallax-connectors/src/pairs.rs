//! Cross-language transmute / mirror pair maturity.

use crate::catalog::{find, ConnectorMaturity};
use serde::{Deserialize, Serialize};

/// Pair quality for project migration / Mirror sync.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PairMaturity {
    Tier1,
    Tier2,
    Experimental,
    Scaffold,
    Unsupported,
}

impl PairMaturity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tier1 => "tier1",
            Self::Tier2 => "tier2",
            Self::Experimental => "experimental",
            Self::Scaffold => "scaffold",
            Self::Unsupported => "unsupported",
        }
    }
}

/// Resolve pair maturity from connector ids (or aliases).
pub fn pair_maturity(source: &str, target: &str) -> PairMaturity {
    let src = find(source).map(|c| c.id).unwrap_or(source);
    let tgt = find(target).map(|c| c.id).unwrap_or(target);
    let src = normalize_js_family(src);
    let tgt = normalize_js_family(tgt);

    match (src, tgt) {
        ("typescript" | "javascript", "rust") => PairMaturity::Tier1,
        ("python", "rust") => PairMaturity::Tier2,
        ("typescript" | "javascript", "go") => PairMaturity::Tier2,
        ("rust", "typescript" | "javascript") => PairMaturity::Experimental,
        ("python", "typescript" | "javascript") => PairMaturity::Experimental,
        ("typescript" | "javascript", "python") => PairMaturity::Experimental,
        // Same-language "pairs" are identity — not migration.
        (a, b) if a == b => PairMaturity::Unsupported,
        (s, t) => {
            let src_ok = find(s).is_some_and(|c| c.roles.transmute_source);
            let tgt_ok = find(t).is_some_and(|c| c.roles.transmute_target);
            if src_ok && tgt_ok {
                // Both declared as transmute roles → scaffold pair at best.
                let src_m = find(s).map(|c| c.maturity);
                let tgt_m = find(t).map(|c| c.maturity);
                if matches!(
                    (src_m, tgt_m),
                    (
                        Some(ConnectorMaturity::Production | ConnectorMaturity::Experimental),
                        Some(ConnectorMaturity::Production | ConnectorMaturity::Experimental)
                    )
                ) {
                    PairMaturity::Experimental
                } else {
                    PairMaturity::Scaffold
                }
            } else {
                PairMaturity::Unsupported
            }
        }
    }
}

fn normalize_js_family(id: &str) -> &str {
    match id {
        "javascript" | "typescript" => id,
        "js" => "javascript",
        "ts" => "typescript",
        other => other,
    }
}

/// Human summary row for CLI / docs.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PairRow {
    pub source: String,
    pub target: String,
    pub maturity: PairMaturity,
}

/// Notable pairs to highlight (not the full N×N cartesian product).
pub fn highlighted_pairs() -> Vec<PairRow> {
    let specs = [
        ("typescript", "rust"),
        ("javascript", "rust"),
        ("python", "rust"),
        ("typescript", "go"),
        ("rust", "typescript"),
        ("python", "javascript"),
        ("java", "rust"),
        ("csharp", "rust"),
        ("go", "rust"),
        ("ruby", "rust"),
        ("php", "rust"),
        ("kotlin", "rust"),
        ("swift", "rust"),
        ("c", "rust"),
        ("cpp", "rust"),
        ("typescript", "python"),
        ("typescript", "csharp"),
        ("java", "kotlin"),
        ("csharp", "go"),
        ("solidity", "rust"),
    ];
    specs
        .into_iter()
        .map(|(s, t)| PairRow {
            source: s.into(),
            target: t.into(),
            maturity: pair_maturity(s, t),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier1_ts_rust() {
        assert_eq!(pair_maturity("ts", "rust"), PairMaturity::Tier1);
    }

    #[test]
    fn scaffold_java_rust() {
        assert_eq!(pair_maturity("java", "rust"), PairMaturity::Scaffold);
    }
}
