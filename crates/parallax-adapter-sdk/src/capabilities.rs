//! Explicit construct / feature capability flags.

use serde::{Deserialize, Serialize};

/// Support level for a single construct.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilitySupport {
    Full,
    Partial,
    Unsupported,
    Unknown,
}

impl CapabilitySupport {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Full => "FULL",
            Self::Partial => "PARTIAL",
            Self::Unsupported => "UNSUPPORTED",
            Self::Unknown => "UNKNOWN",
        }
    }

    pub fn glyph(self) -> &'static str {
        self.as_str()
    }
}

/// Named capability flag.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityFlag {
    pub name: String,
    pub support: CapabilitySupport,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl CapabilityFlag {
    pub fn new(name: impl Into<String>, support: CapabilitySupport) -> Self {
        Self {
            name: name.into(),
            support,
            detail: None,
        }
    }

    pub fn with_detail(mut self, d: impl Into<String>) -> Self {
        self.detail = Some(d.into());
        self
    }
}

/// Language-construct capability row (for source/target adapters).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConstructCapability {
    pub construct: String,
    pub support: CapabilitySupport,
}

/// Bundle of adapter capabilities (never assume support).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AdapterCapabilities {
    #[serde(default)]
    pub flags: Vec<CapabilityFlag>,
    #[serde(default)]
    pub constructs: Vec<ConstructCapability>,
}

impl AdapterCapabilities {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn with_flag(mut self, name: &str, support: CapabilitySupport) -> Self {
        self.flags.push(CapabilityFlag::new(name, support));
        self
    }

    pub fn with_construct(mut self, name: &str, support: CapabilitySupport) -> Self {
        self.constructs.push(ConstructCapability {
            construct: name.into(),
            support,
        });
        self
    }

    pub fn get(&self, name: &str) -> CapabilitySupport {
        self.flags
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case(name))
            .map(|f| f.support)
            .unwrap_or(CapabilitySupport::Unknown)
    }

    /// TypeScript / JavaScript source profile (honest).
    pub fn typescript_source() -> Self {
        Self::empty()
            .with_flag("parsing", CapabilitySupport::Full)
            .with_flag("types", CapabilitySupport::Full)
            .with_flag("generics", CapabilitySupport::Full)
            .with_flag("decorators", CapabilitySupport::Partial)
            .with_flag("jsx", CapabilitySupport::Full)
            .with_flag("dynamic_eval", CapabilitySupport::Unsupported)
            .with_flag("source_maps", CapabilitySupport::Full)
            .with_flag("control_flow", CapabilitySupport::Full)
            .with_flag("async", CapabilitySupport::Full)
            .with_construct("classes", CapabilitySupport::Full)
            .with_construct("interfaces", CapabilitySupport::Full)
            .with_construct("closures", CapabilitySupport::Full)
    }

    pub fn python_source() -> Self {
        Self::empty()
            .with_flag("parsing", CapabilitySupport::Partial)
            .with_flag("types", CapabilitySupport::Partial)
            .with_flag("async", CapabilitySupport::Partial)
            .with_flag("decorators", CapabilitySupport::Partial)
            .with_flag("dynamic_eval", CapabilitySupport::Unsupported)
            .with_flag("metaprogramming", CapabilitySupport::Unsupported)
            .with_construct("classes", CapabilitySupport::Partial)
            .with_construct("generators", CapabilitySupport::Unsupported)
    }

    pub fn rust_target() -> Self {
        Self::empty()
            .with_flag("codegen", CapabilitySupport::Full)
            .with_flag("ownership", CapabilitySupport::Partial)
            .with_flag("async", CapabilitySupport::Partial)
            .with_flag("error_model", CapabilitySupport::Partial)
            .with_flag("formatting", CapabilitySupport::Full)
            .with_construct("traits", CapabilitySupport::Partial)
            .with_construct("macros", CapabilitySupport::Unsupported)
    }

    pub fn go_target() -> Self {
        Self::empty()
            .with_flag("codegen", CapabilitySupport::Partial)
            .with_flag("async", CapabilitySupport::Partial)
            .with_flag("formatting", CapabilitySupport::Full)
            .with_construct("generics", CapabilitySupport::Partial)
    }

    pub fn framework_http() -> Self {
        Self::empty()
            .with_flag("routes", CapabilitySupport::Full)
            .with_flag("middleware", CapabilitySupport::Partial)
            .with_flag("websocket", CapabilitySupport::Partial)
            .with_flag("sse", CapabilitySupport::Unsupported)
    }

    pub fn scaffold() -> Self {
        Self::empty()
            .with_flag("detection", CapabilitySupport::Partial)
            .with_flag("codegen", CapabilitySupport::Unsupported)
            .with_flag("verification", CapabilitySupport::Unsupported)
    }
}
