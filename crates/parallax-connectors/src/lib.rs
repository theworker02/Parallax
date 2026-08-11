//! Parallax language connector catalog.
//!
//! Registers scaffold runtime adapters for dozens of languages and exposes
//! the transmute/mirror pair matrix. Production adapters (Python, JS, WASM)
//! remain in their dedicated crates — this catalog does not replace them.

#![deny(unsafe_code)]
#![allow(missing_docs)]

mod catalog;
mod pairs;
mod scaffold;
mod worker;

pub use catalog::{
    find, from_extension, production_runtime_ids, scaffold_runtime_connectors, ConnectorDef,
    ConnectorMaturity, ConnectorRoles, LanguageFamily, CONNECTORS,
};
pub use pairs::{highlighted_pairs, pair_maturity, PairMaturity, PairRow};

use parallax_runtime::RuntimeManager;
use tracing::debug;

/// Register experimental workers (when hosts exist) plus scaffolds for everything else.
pub fn register_all_lenient(manager: &RuntimeManager) {
    let workers = worker::register_experimental_workers(manager);
    let mut n = 0usize;
    for def in scaffold_runtime_connectors() {
        if workers.contains(&def.id) {
            continue;
        }
        manager.register(scaffold::ScaffoldAdapter::boxed(def));
        n += 1;
        debug!(id = def.id, maturity = def.maturity.as_str(), "registered scaffold connector");
    }
    debug!(
        scaffolds = n,
        workers = workers.len(),
        "parallax-connectors registration complete"
    );
}

/// JSON-serializable catalog snapshot for `plx connectors`.
pub fn catalog_snapshot() -> CatalogSnapshot {
    CatalogSnapshot {
        version: 1,
        count: CONNECTORS.len(),
        connectors: CONNECTORS
            .iter()
            .map(|c| ConnectorInfo {
                id: c.id.to_string(),
                name: c.name.to_string(),
                aliases: c.aliases.iter().map(|s| (*s).to_string()).collect(),
                extensions: c.extensions.iter().map(|s| (*s).to_string()).collect(),
                family: c.family.as_str().to_string(),
                maturity: c.maturity.as_str().to_string(),
                runtime: c.roles.runtime,
                value_migrate: c.roles.value_migrate,
                transmute_source: c.roles.transmute_source,
                transmute_target: c.roles.transmute_target,
                host_binaries: c.host_binaries.iter().map(|s| (*s).to_string()).collect(),
                notes: c.notes.to_string(),
            })
            .collect(),
        highlighted_pairs: highlighted_pairs(),
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CatalogSnapshot {
    pub version: u32,
    pub count: usize,
    pub connectors: Vec<ConnectorInfo>,
    pub highlighted_pairs: Vec<PairRow>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectorInfo {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub extensions: Vec<String>,
    pub family: String,
    pub maturity: String,
    pub runtime: bool,
    pub value_migrate: bool,
    pub transmute_source: bool,
    pub transmute_target: bool,
    pub host_binaries: Vec<String>,
    pub notes: String,
}
