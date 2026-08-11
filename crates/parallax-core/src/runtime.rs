//! Runtime identity and metadata.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported runtime kinds. Extensible without coupling core to adapters.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeKind {
    /// CPython (or compatible) interpreter.
    Python,
    /// JavaScript via Node.js (or compatible engine).
    #[serde(rename = "javascript")]
    JavaScript,
    /// WebAssembly via a sandboxed engine.
    Wasm,
    /// Reserved for future adapters.
    Other(String),
}

impl RuntimeKind {
    /// Canonical short name used by the CLI.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Python => "python",
            Self::JavaScript => "javascript",
            Self::Wasm => "wasm",
            Self::Other(name) => name.as_str(),
        }
    }

    /// Parse a runtime name from CLI / config input.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "python" | "py" | "cpython" => Some(Self::Python),
            "javascript" | "js" | "node" | "nodejs" => Some(Self::JavaScript),
            "wasm" | "webassembly" | "wat" => Some(Self::Wasm),
            other if !other.is_empty() => Some(Self::Other(other.to_string())),
            _ => None,
        }
    }

    /// File extensions typically associated with this runtime.
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Self::Python => &["py"],
            Self::JavaScript => &["js", "mjs", "cjs"],
            Self::Wasm => &["wasm", "wat"],
            Self::Other(_) => &[],
        }
    }

    /// Infer runtime from a file path extension.
    ///
    /// First-class production runtimes map to named variants; everything else
    /// becomes [`RuntimeKind::Other`] with a canonical connector id when known.
    pub fn from_path(path: &std::path::Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        match ext.as_str() {
            "py" => Some(Self::Python),
            "js" | "mjs" | "cjs" | "jsx" => Some(Self::JavaScript),
            "wasm" | "wat" => Some(Self::Wasm),
            // Broad connector map (canonical ids used by parallax-connectors).
            "ts" | "tsx" => Some(Self::Other("typescript".into())),
            "go" => Some(Self::Other("go".into())),
            "rs" => Some(Self::Other("rust".into())),
            "java" => Some(Self::Other("java".into())),
            "kt" | "kts" => Some(Self::Other("kotlin".into())),
            "scala" | "sc" => Some(Self::Other("scala".into())),
            "cs" | "csx" => Some(Self::Other("csharp".into())),
            "fs" | "fsx" => Some(Self::Other("fsharp".into())),
            "rb" => Some(Self::Other("ruby".into())),
            "php" => Some(Self::Other("php".into())),
            "pl" | "pm" => Some(Self::Other("perl".into())),
            "lua" => Some(Self::Other("lua".into())),
            "r" => Some(Self::Other("r".into())),
            "jl" => Some(Self::Other("julia".into())),
            "swift" => Some(Self::Other("swift".into())),
            "dart" => Some(Self::Other("dart".into())),
            "ex" | "exs" => Some(Self::Other("elixir".into())),
            "erl" | "hrl" => Some(Self::Other("erlang".into())),
            "hs" | "lhs" => Some(Self::Other("haskell".into())),
            "ml" | "mli" => Some(Self::Other("ocaml".into())),
            "clj" | "cljs" | "cljc" => Some(Self::Other("clojure".into())),
            "c" | "h" => Some(Self::Other("c".into())),
            "cpp" | "cc" | "cxx" | "hpp" | "hh" => Some(Self::Other("cpp".into())),
            "zig" => Some(Self::Other("zig".into())),
            "nim" => Some(Self::Other("nim".into())),
            "cr" => Some(Self::Other("crystal".into())),
            "sol" => Some(Self::Other("solidity".into())),
            "ps1" | "psm1" => Some(Self::Other("powershell".into())),
            "sh" | "bash" | "zsh" => Some(Self::Other("shell".into())),
            "sql" => Some(Self::Other("sql".into())),
            _ => None,
        }
    }
}

impl fmt::Display for RuntimeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// High-level availability of a runtime on the host.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeStatus {
    /// Runtime binary/engine is present and responsive.
    Ready,
    /// Runtime is installed but failed a health check.
    Degraded {
        /// Human-readable reason.
        reason: String,
    },
    /// Runtime is not available on this host.
    Unavailable {
        /// Human-readable reason.
        reason: String,
    },
}

/// Static metadata describing a runtime adapter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeMetadata {
    /// Runtime kind.
    pub kind: RuntimeKind,
    /// Human-readable display name.
    pub name: String,
    /// Adapter implementation version.
    pub adapter_version: String,
    /// Detected host runtime version, if known.
    pub host_version: Option<String>,
    /// Adapter interface version this implementation targets.
    pub interface_version: u32,
    /// Short description.
    pub description: String,
}

impl RuntimeMetadata {
    /// Construct metadata for a built-in adapter.
    pub fn builtin(
        kind: RuntimeKind,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            name: name.into(),
            adapter_version: crate::PARALLAX_VERSION.to_string(),
            host_version: None,
            interface_version: crate::ADAPTER_INTERFACE_VERSION,
            description: description.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_aliases() {
        assert_eq!(RuntimeKind::parse("py"), Some(RuntimeKind::Python));
        assert_eq!(RuntimeKind::parse("node"), Some(RuntimeKind::JavaScript));
        assert_eq!(RuntimeKind::parse("wat"), Some(RuntimeKind::Wasm));
    }

    #[test]
    fn from_path() {
        assert_eq!(
            RuntimeKind::from_path(std::path::Path::new("demo.py")),
            Some(RuntimeKind::Python)
        );
    }
}
