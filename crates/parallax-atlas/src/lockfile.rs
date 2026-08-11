//! Reproducible adapter lockfile (`parallax.lock`).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::registry::AdapterRegistry;
use crate::ATLAS_FORMAT_VERSION;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LockEntry {
    pub id: String,
    pub version: String,
    pub maturity: String,
    pub adapter_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdapterLockfile {
    pub format_version: u32,
    pub generated_at: DateTime<Utc>,
    pub parallax_version: String,
    pub adapters: Vec<LockEntry>,
}

impl AdapterLockfile {
    pub fn from_registry(reg: &AdapterRegistry) -> Self {
        let mut adapters: Vec<_> = reg
            .list()
            .into_iter()
            .map(|m| LockEntry {
                id: m.id.0,
                version: m.version,
                maturity: m.maturity.as_str().to_string(),
                adapter_type: m.adapter_type.as_str().to_string(),
            })
            .collect();
        adapters.sort_by(|a, b| a.id.cmp(&b.id));
        Self {
            format_version: ATLAS_FORMAT_VERSION,
            generated_at: Utc::now(),
            parallax_version: parallax_core::PARALLAX_VERSION.to_string(),
            adapters,
        }
    }

    pub fn write(&self, path: &Path) -> Result<(), parallax_core::ParallaxError> {
        let text = serde_json::to_string_pretty(self).map_err(|e| {
            parallax_core::ParallaxError::new(
                parallax_core::ErrorCode::SerializationFailure,
                e.to_string(),
            )
        })?;
        fs::write(path, text).map_err(|e| {
            parallax_core::ParallaxError::new(parallax_core::ErrorCode::Io, e.to_string())
        })?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, parallax_core::ParallaxError> {
        let text = fs::read_to_string(path).map_err(|e| {
            parallax_core::ParallaxError::new(parallax_core::ErrorCode::Io, e.to_string())
        })?;
        serde_json::from_str(&text).map_err(|e| {
            parallax_core::ParallaxError::new(
                parallax_core::ErrorCode::SerializationFailure,
                e.to_string(),
            )
        })
    }
}
