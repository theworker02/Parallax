//! PIR document: bindings + optional object graph.

use crate::value::PirValue;
use crate::Result;
use indexmap::IndexMap;
use parallax_core::{check_pir_schema, ErrorCode, ObjectId, ParallaxError, PIR_SCHEMA_VERSION};
use serde::{Deserialize, Serialize};

/// Named root pointer into the object graph.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PirRoot {
    /// Root name (e.g. "globals", "locals").
    pub name: String,
    /// Object id.
    pub id: ObjectId,
}

/// A named top-level binding captured from a guest runtime.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PirBinding {
    /// Binding name.
    pub name: String,
    /// Bound value.
    pub value: PirValue,
}

/// Complete PIR document.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PirDocument {
    /// Schema version.
    pub schema: u32,
    /// Top-level named bindings (primary migration surface).
    pub bindings: IndexMap<String, PirValue>,
    /// Optional heap objects keyed by id.
    pub objects: IndexMap<u64, PirValue>,
    /// Named roots into `objects`.
    pub roots: Vec<PirRoot>,
    /// Free-form metadata.
    pub metadata: IndexMap<String, serde_json::Value>,
}

impl Default for PirDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl PirDocument {
    /// Empty document at current schema.
    pub fn new() -> Self {
        Self {
            schema: PIR_SCHEMA_VERSION,
            bindings: IndexMap::new(),
            objects: IndexMap::new(),
            roots: Vec::new(),
            metadata: IndexMap::new(),
        }
    }

    /// Set or replace a binding.
    pub fn set_binding(&mut self, name: impl Into<String>, value: PirValue) {
        self.bindings.insert(name.into(), value);
    }

    /// Get a binding by name.
    pub fn binding(&self, name: &str) -> Option<&PirValue> {
        self.bindings.get(name)
    }

    /// Validate schema and internal consistency.
    pub fn validate(&self) -> Result<()> {
        check_pir_schema(self.schema)
            .map_err(|e| e.with_source("parallax-ir").with_operation("validate"))?;
        for root in &self.roots {
            if !self.objects.contains_key(&root.id.raw()) {
                return Err(ParallaxError::new(
                    ErrorCode::InvalidSnapshot,
                    format!("root '{}' points to missing object {}", root.name, root.id),
                )
                .with_source("parallax-ir")
                .with_operation("validate"));
            }
        }
        if let Some(id) = self.first_dangling_ref() {
            return Err(ParallaxError::new(
                ErrorCode::InvalidSnapshot,
                format!("dangling PIR ref to missing object {id}"),
            )
            .with_source("parallax-ir")
            .with_operation("validate"));
        }
        Ok(())
    }

    fn first_dangling_ref(&self) -> Option<ObjectId> {
        use std::cell::Cell;
        let found = Cell::new(None);
        let objects = &self.objects;
        let mut visit = |v: &PirValue| {
            if found.get().is_some() {
                return;
            }
            if let PirValue::Ref { id } = v {
                if !objects.contains_key(&id.raw()) {
                    found.set(Some(*id));
                }
            }
        };
        for v in self.bindings.values() {
            v.walk(&mut visit);
            if found.get().is_some() {
                break;
            }
        }
        if found.get().is_none() {
            for v in self.objects.values() {
                v.walk(&mut visit);
                if found.get().is_some() {
                    break;
                }
            }
        }
        found.get()
    }

    /// Count all values in bindings (shallow + nested nodes).
    pub fn value_count(&self) -> usize {
        let mut n = 0usize;
        for v in self.bindings.values() {
            v.walk(&mut |_| n += 1);
        }
        for v in self.objects.values() {
            v.walk(&mut |_| n += 1);
        }
        n
    }

    /// Build a document from a JSON object of bindings in PIR tagged form.
    pub fn from_bindings_json(bindings: &serde_json::Value) -> Result<Self> {
        let mut doc = Self::new();
        let obj = bindings.as_object().ok_or_else(|| {
            ParallaxError::new(
                ErrorCode::SerializationFailure,
                "bindings must be an object",
            )
            .with_source("parallax-ir")
            .with_operation("from_bindings_json")
        })?;
        for (k, v) in obj {
            doc.set_binding(k.clone(), PirValue::from_json(v)?);
        }
        Ok(doc)
    }

    /// Export bindings as tagged JSON object.
    pub fn bindings_to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        for (k, v) in &self.bindings {
            map.insert(k.clone(), v.to_json());
        }
        serde_json::Value::Object(map)
    }
}
