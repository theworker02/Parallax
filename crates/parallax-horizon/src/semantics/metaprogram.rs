//! Metaprogram expansion and decorator lowering.

#![deny(unsafe_code)]
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExpandedMethod {
    pub name: String,
    pub origin: String,
    pub notes: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoweredDecorator {
    pub source: String,
    pub effect: String,
    pub target_equivalent: String,
}

#[derive(Clone, Debug, Default)]
pub struct MetaprogramExpander;

impl MetaprogramExpander {
    pub fn expand_define_method_prefix(
        &self,
        prefix: &str,
        observed_suffixes: &[&str],
    ) -> Vec<ExpandedMethod> {
        observed_suffixes
            .iter()
            .map(|s| ExpandedMethod {
                name: format!("{prefix}{s}"),
                origin: format!("define_method / method_missing prefix `{prefix}`"),
                notes: "Expanded at analysis time into concrete PUIR functions".into(),
            })
            .collect()
    }

    pub fn lower_decorator(&self, name: &str) -> LoweredDecorator {
        match name {
            "dataclass" | "dataclass()" => LoweredDecorator {
                source: name.into(),
                effect: "generate constructor/eq/repr fields".into(),
                target_equivalent: "struct + derive(Debug, Clone, PartialEq)".into(),
            },
            "Entity" => LoweredDecorator {
                source: name.into(),
                effect: "persistence entity mapping".into(),
                target_equivalent: "ORM model / table mapping".into(),
            },
            other => LoweredDecorator {
                source: other.into(),
                effect: "unknown — preserve via capsule or manual".into(),
                target_equivalent: "manual review".into(),
            },
        }
    }
}
