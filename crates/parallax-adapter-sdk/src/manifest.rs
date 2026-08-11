//! Adapter identity and manifest.

use serde::{Deserialize, Serialize};

/// Stable adapter identifier (`parallax.typescript.source`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AdapterId(pub String);

impl AdapterId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AdapterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Adapter category.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdapterKind {
    SourceLanguage,
    TargetLanguage,
    Framework,
    Dependency,
    BuildSystem,
    TestFramework,
    Database,
    Orm,
    Runtime,
    Configuration,
    Deployment,
    Verification,
    Formatter,
    Linter,
    Serialization,
    Validation,
    CliFramework,
    WebFrontend,
    DesktopGui,
    Codegen,
    PairProfile,
    Other,
}

impl AdapterKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SourceLanguage => "source-language",
            Self::TargetLanguage => "target-language",
            Self::Framework => "framework",
            Self::Dependency => "dependency",
            Self::BuildSystem => "build-system",
            Self::TestFramework => "test-framework",
            Self::Database => "database",
            Self::Orm => "orm",
            Self::Runtime => "runtime",
            Self::Configuration => "configuration",
            Self::Deployment => "deployment",
            Self::Verification => "verification",
            Self::Formatter => "formatter",
            Self::Linter => "linter",
            Self::Serialization => "serialization",
            Self::Validation => "validation",
            Self::CliFramework => "cli-framework",
            Self::WebFrontend => "web-frontend",
            Self::DesktopGui => "desktop-gui",
            Self::Codegen => "codegen",
            Self::PairProfile => "pair-profile",
            Self::Other => "other",
        }
    }
}

/// Declared maturity / conformance intent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterMaturity {
    Stable,
    Beta,
    Experimental,
    ParseOnly,
    TargetOnly,
    Scaffold,
}

impl AdapterMaturity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Beta => "beta",
            Self::Experimental => "experimental",
            Self::ParseOnly => "parse_only",
            Self::TargetOnly => "target_only",
            Self::Scaffold => "scaffold",
        }
    }
}

/// Conformance medal (Bronze / Silver / Gold).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConformanceLevel {
    #[default]
    None,
    Bronze,
    Silver,
    Gold,
}

impl ConformanceLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Bronze => "bronze",
            Self::Silver => "silver",
            Self::Gold => "gold",
        }
    }
}

/// Author metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdapterAuthor {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Capability-based host permissions for third-party adapters.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AdapterPermissions {
    pub read_project: bool,
    pub write_output: bool,
    pub execute_build: bool,
    pub network: bool,
    pub read_environment: bool,
}

impl AdapterPermissions {
    pub fn minimal() -> Self {
        Self {
            read_project: true,
            write_output: false,
            execute_build: false,
            network: false,
            read_environment: false,
        }
    }

    pub fn builtin_full() -> Self {
        Self {
            read_project: true,
            write_output: true,
            execute_build: true,
            network: false,
            read_environment: true,
        }
    }
}

/// Machine-readable adapter declaration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdapterManifest {
    pub id: AdapterId,
    pub name: String,
    pub version: String,
    pub adapter_type: AdapterKind,
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub ecosystems: Vec<String>,
    pub maturity: AdapterMaturity,
    #[serde(default)]
    pub conformance: ConformanceLevel,
    #[serde(default)]
    pub depends_on: Vec<AdapterId>,
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub owns: Vec<String>,
    #[serde(default)]
    pub permissions: AdapterPermissions,
    #[serde(default)]
    pub author: Option<AdapterAuthor>,
    #[serde(default)]
    pub notes: String,
    /// Independent adapter schema version this manifest targets.
    #[serde(default = "default_sdk")]
    pub sdk_version: u32,
}

fn default_sdk() -> u32 {
    crate::ADAPTER_SDK_VERSION
}

impl AdapterManifest {
    pub fn builtin(
        id: &str,
        name: &str,
        kind: AdapterKind,
        maturity: AdapterMaturity,
        languages: &[&str],
    ) -> Self {
        Self {
            id: AdapterId::new(id),
            name: name.into(),
            version: parallax_core::PARALLAX_VERSION.to_string(),
            adapter_type: kind,
            languages: languages.iter().map(|s| (*s).to_string()).collect(),
            ecosystems: Vec::new(),
            maturity,
            conformance: match maturity {
                AdapterMaturity::Stable => ConformanceLevel::Gold,
                AdapterMaturity::Beta => ConformanceLevel::Silver,
                AdapterMaturity::Experimental => ConformanceLevel::Bronze,
                _ => ConformanceLevel::None,
            },
            depends_on: Vec::new(),
            priority: 0,
            owns: Vec::new(),
            permissions: AdapterPermissions::builtin_full(),
            author: Some(AdapterAuthor {
                name: "Parallax".into(),
                url: Some("https://github.com/parallax-runtime/parallax".into()),
            }),
            notes: String::new(),
            sdk_version: crate::ADAPTER_SDK_VERSION,
        }
    }
}
