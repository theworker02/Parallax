//! Dynamic type crystallization — closed unions from observations.

#![deny(unsafe_code)]
#![allow(missing_docs)]

use super::shape_inference::{FieldType, Shape};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrystallizedType {
    pub name: String,
    pub kind: CrystallizedKind,
    pub rust_sketch: String,
    pub confidence: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrystallizedKind {
    Struct,
    Enum,
    Newtype,
    ErrorEnum,
}

#[derive(Clone, Debug, Default)]
pub struct TypeCrystallizer;

impl TypeCrystallizer {
    pub fn from_variants(&self, name: &str, variants: &[&str]) -> CrystallizedType {
        let body = variants
            .iter()
            .map(|v| format!("    {v}({v}),"))
            .collect::<Vec<_>>()
            .join("\n");
        CrystallizedType {
            rust_sketch: format!("enum {name} {{\n{body}\n}}"),
            name: name.into(),
            kind: CrystallizedKind::Enum,
            confidence: 0.8,
        }
    }

    pub fn from_shape(&self, shape: &Shape) -> CrystallizedType {
        let mut fields = String::new();
        for (k, v) in &shape.fields {
            let ty = match v {
                FieldType::String => "String",
                FieldType::OptionalString => "Option<String>",
                FieldType::Bool => "bool",
                FieldType::Int => "i64",
                FieldType::Float => "f64",
                FieldType::List(_) => "Vec<String>",
                FieldType::Nested(n) => n.as_str(),
                FieldType::Unknown => "serde_json::Value",
            };
            fields.push_str(&format!("    pub {k}: {ty},\n"));
        }
        CrystallizedType {
            rust_sketch: format!("struct {} {{\n{fields}}}", shape.name),
            name: shape.name.clone(),
            kind: CrystallizedKind::Struct,
            confidence: 0.75,
        }
    }

    pub fn invent_errors(&self, name: &str, messages: &[&str]) -> CrystallizedType {
        let vars: Vec<String> = messages
            .iter()
            .map(|m| {
                let v = m
                    .split_whitespace()
                    .map(|w| {
                        let mut c = w.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                        }
                    })
                    .collect::<String>()
                    .chars()
                    .filter(|c| c.is_ascii_alphanumeric())
                    .collect::<String>();
                if v.is_empty() {
                    "Unknown".into()
                } else {
                    v
                }
            })
            .collect();
        let body = vars
            .iter()
            .map(|v| format!("    {v},"))
            .collect::<Vec<_>>()
            .join("\n");
        CrystallizedType {
            rust_sketch: format!("enum {name} {{\n{body}\n}}"),
            name: name.into(),
            kind: CrystallizedKind::ErrorEnum,
            confidence: 0.7,
        }
    }
}
