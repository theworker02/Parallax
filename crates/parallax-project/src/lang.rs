//! Source / target language identity for Transmute.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Source language frontend.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceLanguage {
    /// Python.
    Python,
    /// TypeScript.
    TypeScript,
    /// JavaScript.
    JavaScript,
    /// Rust.
    Rust,
    /// Go.
    Go,
    /// Other / unknown.
    Other(String),
}

impl SourceLanguage {
    /// CLI name.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Python => "python",
            Self::TypeScript => "typescript",
            Self::JavaScript => "javascript",
            Self::Rust => "rust",
            Self::Go => "go",
            Self::Other(s) => s.as_str(),
        }
    }

    /// Parse CLI / config name.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "python" | "py" => Some(Self::Python),
            "typescript" | "ts" => Some(Self::TypeScript),
            "javascript" | "js" | "node" => Some(Self::JavaScript),
            "rust" | "rs" => Some(Self::Rust),
            "go" | "golang" => Some(Self::Go),
            // Long-tail languages use Other with a canonical connector id.
            other if !other.is_empty() => Some(Self::Other(other.to_string())),
            _ => None,
        }
    }
}

impl std::fmt::Display for SourceLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Target language backend.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetLanguage {
    /// Rust.
    Rust,
    /// Go.
    Go,
    /// Python.
    Python,
    /// TypeScript.
    TypeScript,
    /// JavaScript.
    JavaScript,
    /// Other.
    Other(String),
}

impl TargetLanguage {
    /// CLI name.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Rust => "rust",
            Self::Go => "go",
            Self::Python => "python",
            Self::TypeScript => "typescript",
            Self::JavaScript => "javascript",
            Self::Other(s) => s.as_str(),
        }
    }

    /// Parse.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "rust" | "rs" => Some(Self::Rust),
            "go" | "golang" => Some(Self::Go),
            "python" | "py" => Some(Self::Python),
            "typescript" | "ts" => Some(Self::TypeScript),
            "javascript" | "js" => Some(Self::JavaScript),
            other if !other.is_empty() => Some(Self::Other(other.to_string())),
            _ => None,
        }
    }
}

impl std::fmt::Display for TargetLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Detect language mix from a list of relative file paths.
/// Returns percentages totaling ~100 based on counted source files.
pub fn detect_languages(paths: &[impl AsRef<Path>]) -> (Option<SourceLanguage>, HashMap<String, f64>) {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for p in paths {
        let ext = p
            .as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let lang = match ext.as_str() {
            "ts" | "tsx" => "typescript",
            "js" | "jsx" | "mjs" | "cjs" => "javascript",
            "py" | "pyi" => "python",
            "rs" => "rust",
            "go" => "go",
            "java" => "java",
            "kt" | "kts" => "kotlin",
            "scala" | "sc" => "scala",
            "cs" | "csx" => "csharp",
            "fs" | "fsx" => "fsharp",
            "rb" => "ruby",
            "php" => "php",
            "pl" | "pm" => "perl",
            "lua" => "lua",
            "r" => "r",
            "jl" => "julia",
            "swift" => "swift",
            "dart" => "dart",
            "ex" | "exs" => "elixir",
            "erl" | "hrl" => "erlang",
            "hs" | "lhs" => "haskell",
            "ml" | "mli" => "ocaml",
            "clj" | "cljs" | "cljc" => "clojure",
            "c" | "h" => "c",
            "cpp" | "cc" | "cxx" | "hpp" | "hh" => "cpp",
            "zig" => "zig",
            "nim" => "nim",
            "cr" => "crystal",
            "sol" => "solidity",
            "sql" => "sql",
            "sh" | "bash" | "zsh" => "shell",
            "ps1" | "psm1" => "powershell",
            "html" | "htm" => "html",
            "css" => "css",
            _ => continue,
        };
        *counts.entry(lang.to_string()).or_default() += 1;
    }
    let total: usize = counts.values().sum();
    let mut mix = HashMap::new();
    if total > 0 {
        for (k, v) in &counts {
            mix.insert(k.clone(), (*v as f64) * 100.0 / total as f64);
        }
    }
    // Prefer application languages over markup when scoring primary.
    let priority = [
        "typescript",
        "javascript",
        "python",
        "rust",
        "go",
        "java",
        "kotlin",
        "csharp",
        "ruby",
        "php",
        "swift",
        "dart",
        "c",
        "cpp",
        "scala",
        "elixir",
        "haskell",
    ];
    let primary = priority
        .iter()
        .filter_map(|k| counts.get(*k).map(|c| (*k, *c)))
        .max_by_key(|(_, c)| *c)
        .and_then(|(k, _)| SourceLanguage::parse(k));
    (primary, mix)
}
