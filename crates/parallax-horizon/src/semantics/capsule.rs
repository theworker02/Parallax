//! Compatibility capsules — smallest semantic surface required by a project.

#![deny(unsafe_code)]
#![allow(missing_docs)]

use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;

/// Capability a capsule may implement.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapsuleCapability {
    AttributeLookup,
    AttributeStore,
    RuntimeStringKeys,
    OptionalCallableMembers,
    OpenDispatch,
    PatchTable,
    NullishCoercion,
    Truthiness,
    DynamicImport,
    Eval,
    Descriptors,
    Metaclasses,
}

impl CapsuleCapability {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AttributeLookup => "attribute_lookup",
            Self::AttributeStore => "attribute_store",
            Self::RuntimeStringKeys => "runtime_string_keys",
            Self::OptionalCallableMembers => "optional_callable_members",
            Self::OpenDispatch => "open_dispatch",
            Self::PatchTable => "patch_table",
            Self::NullishCoercion => "nullish_coercion",
            Self::Truthiness => "truthiness",
            Self::DynamicImport => "dynamic_import",
            Self::Eval => "eval",
            Self::Descriptors => "descriptors",
            Self::Metaclasses => "metaclasses",
        }
    }
}

/// Minified set of capabilities actually required.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CapsuleSpec {
    pub name: String,
    pub target_language: String,
    pub capabilities: IndexSet<CapsuleCapability>,
    pub excluded: IndexSet<CapsuleCapability>,
    pub notes: String,
}

/// Semantic minifier: source usage → minimal capsule.
#[derive(Clone, Debug, Default)]
pub struct SemanticMinifier;

impl SemanticMinifier {
    pub fn minify(&self, name: &str, target: &str, required: &[CapsuleCapability]) -> CapsuleSpec {
        let mut capabilities = IndexSet::new();
        for c in required {
            capabilities.insert(c.clone());
        }
        let all = [
            CapsuleCapability::AttributeLookup,
            CapsuleCapability::AttributeStore,
            CapsuleCapability::RuntimeStringKeys,
            CapsuleCapability::OptionalCallableMembers,
            CapsuleCapability::OpenDispatch,
            CapsuleCapability::PatchTable,
            CapsuleCapability::NullishCoercion,
            CapsuleCapability::Truthiness,
            CapsuleCapability::DynamicImport,
            CapsuleCapability::Eval,
            CapsuleCapability::Descriptors,
            CapsuleCapability::Metaclasses,
        ];
        let mut excluded = IndexSet::new();
        for c in all {
            if !capabilities.contains(&c) {
                excluded.insert(c);
            }
        }
        CapsuleSpec {
            name: name.into(),
            target_language: target.into(),
            capabilities,
            excluded,
            notes: "Specialized capsule — not a full source-language emulator".into(),
        }
    }
}

/// Generated capsule artifact (source text for target project).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedCapsule {
    pub spec: CapsuleSpec,
    pub files: Vec<CapsuleFile>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CapsuleFile {
    pub relative_path: String,
    pub contents: String,
}

/// Capsule generator for Rust targets (honest minimal stubs).
#[derive(Clone, Debug, Default)]
pub struct CapsuleGenerator;

impl CapsuleGenerator {
    pub fn generate_rust(&self, spec: &CapsuleSpec) -> GeneratedCapsule {
        let mut files = Vec::new();
        let mut mod_rs = String::from(
            "//! Parallax compatibility capsule (specialized — not a language emulator).\n\n",
        );
        if spec
            .capabilities
            .contains(&CapsuleCapability::AttributeLookup)
            || spec
                .capabilities
                .contains(&CapsuleCapability::AttributeStore)
        {
            files.push(CapsuleFile {
                relative_path: "parallax_compat/dynamic_object.rs".into(),
                contents: DYNAMIC_OBJECT_RS.into(),
            });
            mod_rs.push_str("pub mod dynamic_object;\n");
        }
        if spec
            .capabilities
            .contains(&CapsuleCapability::RuntimeStringKeys)
        {
            files.push(CapsuleFile {
                relative_path: "parallax_compat/dynamic_value.rs".into(),
                contents: DYNAMIC_VALUE_RS.into(),
            });
            mod_rs.push_str("pub mod dynamic_value;\n");
        }
        if spec
            .capabilities
            .contains(&CapsuleCapability::AttributeLookup)
        {
            files.push(CapsuleFile {
                relative_path: "parallax_compat/attribute_access.rs".into(),
                contents: ATTRIBUTE_ACCESS_RS.into(),
            });
            mod_rs.push_str("pub mod attribute_access;\n");
        }
        if spec
            .capabilities
            .contains(&CapsuleCapability::NullishCoercion)
        {
            files.push(CapsuleFile {
                relative_path: "parallax_compat/nullish.rs".into(),
                contents: NULLISH_RS.into(),
            });
            mod_rs.push_str("pub mod nullish;\n");
        }
        if files.is_empty() {
            mod_rs.push_str("// No capsule modules required for this project.\n");
        }
        let mut header = String::new();
        let _ = writeln!(
            header,
            "// Capsule: {}\n// Target: {}\n// Capabilities: {}\n// Excluded: {}\n",
            spec.name,
            spec.target_language,
            spec.capabilities
                .iter()
                .map(|c| c.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            spec.excluded
                .iter()
                .map(|c| c.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        );
        files.insert(
            0,
            CapsuleFile {
                relative_path: "parallax_compat/mod.rs".into(),
                contents: format!("{header}{mod_rs}"),
            },
        );
        GeneratedCapsule {
            spec: spec.clone(),
            files,
        }
    }
}

const DYNAMIC_OBJECT_RS: &str = r#"//! Minimal dynamic object bag (string-keyed attributes).
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct DynamicObject {
    attrs: HashMap<String, DynamicSlot>,
}

#[derive(Clone, Debug)]
pub enum DynamicSlot {
    Value(String),
    Callable,
    Missing,
}

impl DynamicObject {
    pub fn get(&self, name: &str) -> Option<&DynamicSlot> {
        self.attrs.get(name)
    }
    pub fn set(&mut self, name: impl Into<String>, slot: DynamicSlot) {
        self.attrs.insert(name.into(), slot);
    }
}
"#;

const DYNAMIC_VALUE_RS: &str = r#"//! String-keyed dynamic values for compatibility paths.
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum DynamicValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Map(HashMap<String, DynamicValue>),
    List(Vec<DynamicValue>),
}
"#;

const ATTRIBUTE_ACCESS_RS: &str = r#"//! getattr/setattr-style helpers over DynamicObject.
use super::dynamic_object::{DynamicObject, DynamicSlot};

pub fn getattr(obj: &DynamicObject, name: &str) -> Option<DynamicSlot> {
    obj.get(name).cloned()
}

pub fn setattr(obj: &mut DynamicObject, name: &str, slot: DynamicSlot) {
    obj.set(name, slot);
}
"#;

const NULLISH_RS: &str = r#"//! JS `== null` semantics: null OR undefined.
#[inline]
pub fn is_nullish<T>(v: &Option<T>) -> bool {
    v.is_none()
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minify_excludes_unused() {
        let m = SemanticMinifier;
        let spec = m.minify(
            "py-dynamic",
            "rust",
            &[
                CapsuleCapability::AttributeLookup,
                CapsuleCapability::RuntimeStringKeys,
            ],
        );
        assert!(spec
            .capabilities
            .contains(&CapsuleCapability::AttributeLookup));
        assert!(spec.excluded.contains(&CapsuleCapability::Eval));
        assert!(spec.excluded.contains(&CapsuleCapability::Metaclasses));
    }

    #[test]
    fn generate_rust_files() {
        let spec = SemanticMinifier.minify(
            "demo",
            "rust",
            &[
                CapsuleCapability::AttributeLookup,
                CapsuleCapability::AttributeStore,
            ],
        );
        let gen = CapsuleGenerator.generate_rust(&spec);
        assert!(gen
            .files
            .iter()
            .any(|f| f.relative_path.contains("dynamic_object")));
    }
}
