//! Structural shape inference for dynamic objects.

#![deny(unsafe_code)]
#![allow(missing_docs)]

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shape {
    pub name: String,
    pub fields: IndexMap<String, FieldType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    String,
    OptionalString,
    Bool,
    Int,
    Float,
    List(Box<FieldType>),
    Nested(String),
    Unknown,
}

#[derive(Clone, Debug, Default)]
pub struct ShapeInferencer;

impl ShapeInferencer {
    pub fn from_accesses(&self, name: &str, fields: &[&str]) -> Shape {
        let mut map = IndexMap::new();
        for f in fields {
            let ty = if f.contains("email") || f.contains("name") {
                FieldType::OptionalString
            } else if f.contains("id") {
                FieldType::String
            } else if f.ends_with('s') {
                FieldType::List(Box::new(FieldType::String))
            } else {
                FieldType::Unknown
            };
            map.insert((*f).to_string(), ty);
        }
        Shape {
            name: name.into(),
            fields: map,
        }
    }
}
