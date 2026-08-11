//! Adapter registry.

use parallax_adapter_sdk::{
    AdapterCapabilities, AdapterId, AdapterKind, AdapterManifest, AdapterMaturity, DetectionResult,
    ParallaxAdapter, ProjectContext,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Registered adapter handle.
#[derive(Clone)]
pub struct AdapterEntry {
    pub adapter: Arc<dyn ParallaxAdapter>,
}

/// Detection outcome tied to an adapter id.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegisteredDetection {
    pub id: String,
    pub name: String,
    pub adapter_type: String,
    pub maturity: String,
    pub detection: DetectionResult,
    pub priority: i32,
}

/// Central adapter catalog for Atlas.
pub struct AdapterRegistry {
    adapters: HashMap<String, AdapterEntry>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    pub fn register(&mut self, adapter: Arc<dyn ParallaxAdapter>) {
        let id = adapter.manifest().id.0.clone();
        self.adapters.insert(id, AdapterEntry { adapter });
    }

    pub fn get(&self, id: &str) -> Option<&AdapterEntry> {
        self.adapters.get(id).or_else(|| {
            self.adapters.values().find(|e| {
                let m = e.adapter.manifest();
                m.id.as_str() == id
                    || m.name.eq_ignore_ascii_case(id)
                    || m.languages.iter().any(|l| l.eq_ignore_ascii_case(id))
            })
        })
    }

    pub fn list(&self) -> Vec<AdapterManifest> {
        let mut v: Vec<_> = self
            .adapters
            .values()
            .map(|e| e.adapter.manifest())
            .collect();
        v.sort_by(|a, b| {
            a.adapter_type
                .as_str()
                .cmp(b.adapter_type.as_str())
                .then(a.id.as_str().cmp(b.id.as_str()))
        });
        v
    }

    pub fn list_by_kind(&self, kind: AdapterKind) -> Vec<AdapterManifest> {
        self.list()
            .into_iter()
            .filter(|m| m.adapter_type == kind)
            .collect()
    }

    pub fn capabilities_of(&self, id: &str) -> Option<AdapterCapabilities> {
        self.get(id).map(|e| e.adapter.capabilities())
    }

    pub fn detect_all(&self, ctx: &ProjectContext) -> Vec<RegisteredDetection> {
        let mut out = Vec::new();
        for e in self.adapters.values() {
            let m = e.adapter.manifest();
            let det = e.adapter.detect(ctx);
            if det.matched {
                out.push(RegisteredDetection {
                    id: m.id.0.clone(),
                    name: m.name.clone(),
                    adapter_type: m.adapter_type.as_str().to_string(),
                    maturity: m.maturity.as_str().to_string(),
                    priority: m.priority,
                    detection: det,
                });
            }
        }
        // More specific (higher priority) first; then confidence.
        out.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then(
                    b.detection
                        .confidence
                        .score()
                        .cmp(&a.detection.confidence.score()),
                )
                .then(a.id.cmp(&b.id))
        });
        out
    }

    pub fn resolve_conflicts(&self, detections: &[RegisteredDetection]) -> Vec<RegisteredDetection> {
        // Within the same adapter_type, keep highest priority / confidence.
        let mut best: HashMap<String, RegisteredDetection> = HashMap::new();
        for d in detections {
            best.entry(d.adapter_type.clone())
                .and_modify(|existing| {
                    if d.priority > existing.priority
                        || (d.priority == existing.priority
                            && d.detection.confidence.score()
                                > existing.detection.confidence.score())
                    {
                        *existing = d.clone();
                    }
                })
                .or_insert_with(|| d.clone());
        }
        let mut v: Vec<_> = best.into_values().collect();
        v.sort_by(|a, b| a.adapter_type.cmp(&b.adapter_type).then(a.id.cmp(&b.id)));
        v
    }

    pub fn health_scores(&self) -> Vec<(String, u8)> {
        self.list()
            .into_iter()
            .map(|m| {
                let score: u8 = match m.maturity {
                    AdapterMaturity::Stable => 96,
                    AdapterMaturity::Beta => 82,
                    AdapterMaturity::Experimental => 64,
                    AdapterMaturity::ParseOnly => 55,
                    AdapterMaturity::TargetOnly => 58,
                    AdapterMaturity::Scaffold => 40,
                };
                // Slight boost for gold conformance.
                let score = match m.conformance {
                    parallax_adapter_sdk::ConformanceLevel::Gold => (score + 2).min(100),
                    parallax_adapter_sdk::ConformanceLevel::Silver => score,
                    parallax_adapter_sdk::ConformanceLevel::Bronze => score.saturating_sub(4),
                    parallax_adapter_sdk::ConformanceLevel::None => score.saturating_sub(8),
                };
                (m.id.0, score)
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.adapters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.adapters.is_empty()
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper for dyn adapter from a simple closure-based builtin.
pub struct SimpleAdapter {
    pub manifest: AdapterManifest,
    pub capabilities: AdapterCapabilities,
    pub detect_fn: fn(&ProjectContext) -> DetectionResult,
}

impl ParallaxAdapter for SimpleAdapter {
    fn manifest(&self) -> AdapterManifest {
        self.manifest.clone()
    }

    fn detect(&self, context: &ProjectContext) -> DetectionResult {
        (self.detect_fn)(context)
    }

    fn capabilities(&self) -> AdapterCapabilities {
        self.capabilities.clone()
    }
}

impl SimpleAdapter {
    pub fn arc(self) -> Arc<dyn ParallaxAdapter> {
        Arc::new(self)
    }
}

// silence unused import in some builds
#[allow(dead_code)]
fn _id(s: &str) -> AdapterId {
    AdapterId::new(s)
}
