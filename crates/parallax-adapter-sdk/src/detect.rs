//! Project context and detection results.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Shared analysis context passed to adapters (read-only view).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectContext {
    pub root: PathBuf,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub manifests: HashMap<String, String>,
    #[serde(default)]
    pub language_mix: HashMap<String, f64>,
    #[serde(default)]
    pub packages: Vec<String>,
    #[serde(default)]
    pub hints: HashMap<String, String>,
}

impl ProjectContext {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            files: Vec::new(),
            manifests: HashMap::new(),
            language_mix: HashMap::new(),
            packages: Vec::new(),
            hints: HashMap::new(),
        }
    }

    pub fn has_file_suffix(&self, suffix: &str) -> bool {
        self.files.iter().any(|f| f.ends_with(suffix))
    }

    pub fn has_manifest(&self, name: &str) -> bool {
        self.manifests.contains_key(name)
            || self.files.iter().any(|f| f.ends_with(name) || f == name)
    }

    pub fn package_contains(&self, needle: &str) -> bool {
        let n = needle.to_ascii_lowercase();
        self.packages
            .iter()
            .any(|p| p.to_ascii_lowercase().contains(&n))
    }
}

/// How confident an adapter is that it applies.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionConfidence {
    None,
    Low,
    Medium,
    High,
    Certain,
}

impl DetectionConfidence {
    pub fn score(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Low => 25,
            Self::Medium => 50,
            Self::High => 75,
            Self::Certain => 100,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Certain => "certain",
        }
    }
}

/// Evidence for why an adapter matched.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DetectionEvidence {
    pub kind: String,
    pub detail: String,
}

/// Result of `ParallaxAdapter::detect`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DetectionResult {
    pub matched: bool,
    pub confidence: DetectionConfidence,
    #[serde(default)]
    pub evidence: Vec<DetectionEvidence>,
    #[serde(default)]
    pub owns_nodes: Vec<String>,
}

impl DetectionResult {
    pub fn no_match() -> Self {
        Self {
            matched: false,
            confidence: DetectionConfidence::None,
            evidence: Vec::new(),
            owns_nodes: Vec::new(),
        }
    }

    pub fn matched(confidence: DetectionConfidence) -> Self {
        Self {
            matched: true,
            confidence,
            evidence: Vec::new(),
            owns_nodes: Vec::new(),
        }
    }

    pub fn evidence(mut self, kind: &str, detail: impl Into<String>) -> Self {
        self.evidence.push(DetectionEvidence {
            kind: kind.into(),
            detail: detail.into(),
        });
        self
    }

    pub fn owns(mut self, nodes: &[&str]) -> Self {
        self.owns_nodes
            .extend(nodes.iter().map(|s| (*s).to_string()));
        self
    }
}
