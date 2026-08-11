//! ProjectGraph — authoritative semantic model for migration.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Graph schema version (independent of PUIR).
pub const PROJECT_GRAPH_VERSION: u32 = 1;

/// Relationship / edge kinds.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphEdgeKind {
    /// Calls.
    Calls,
    /// Imports.
    Imports,
    /// Inherits.
    Inherits,
    /// Implements.
    Implements,
    /// Reads.
    Reads,
    /// Writes.
    Writes,
    /// Constructs.
    Constructs,
    /// Returns.
    Returns,
    /// Throws.
    Throws,
    /// Depends on package.
    DependsOn,
    /// Tested by.
    Tests,
    /// Exports.
    Exports,
}

/// Alias used in docs.
pub type Relationship = GraphEdgeKind;

/// Node kinds in the project graph.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphNodeKind {
    /// Module / file unit.
    Module,
    /// Function.
    Function,
    /// Class.
    Class,
    /// Struct / interface type.
    Type,
    /// Package dependency.
    Package,
    /// Test.
    Test,
    /// Entrypoint.
    Entrypoint,
    /// Resource (config, asset).
    Resource,
    /// Configuration file.
    Config,
}

/// A node in the project graph.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphNode {
    /// Stable id.
    pub id: String,
    /// Kind.
    pub kind: GraphNodeKind,
    /// Display name.
    pub name: String,
    /// Owning file if any.
    pub file: Option<String>,
    /// Extra attributes.
    pub attrs: IndexMap<String, serde_json::Value>,
}

/// Directed edge.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphEdge {
    /// From node id.
    pub from: String,
    /// To node id.
    pub to: String,
    /// Kind.
    pub kind: GraphEdgeKind,
}

/// File inventory entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectFile {
    /// Relative path.
    pub path: String,
    /// Role: source | test | config | resource | other
    pub role: String,
    /// Language id if source.
    pub language: Option<String>,
    /// Byte size.
    pub bytes: u64,
}

/// Package / dependency reference.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DependencyRef {
    /// Ecosystem (npm, crates.io, pypi, go).
    pub ecosystem: String,
    /// Package name.
    pub name: String,
    /// Version constraint if known.
    pub version: Option<String>,
    /// Dev dependency?
    pub dev: bool,
}

/// Entrypoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entrypoint {
    /// Relative path.
    pub path: String,
    /// Kind: bin | lib | script
    pub kind: String,
}

/// Complete project graph.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectGraph {
    /// Schema version.
    pub version: u32,
    /// Project name hint.
    pub name: String,
    /// Files.
    pub files: Vec<ProjectFile>,
    /// Packages / dependencies.
    pub packages: Vec<DependencyRef>,
    /// Entrypoints.
    pub entrypoints: Vec<Entrypoint>,
    /// Nodes.
    pub nodes: IndexMap<String, GraphNode>,
    /// Edges.
    pub edges: Vec<GraphEdge>,
    /// Build system hint (npm, cargo, …).
    pub build_system: Option<String>,
    /// Test framework hint.
    pub test_framework: Option<String>,
}

impl ProjectGraph {
    /// Empty graph.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            version: PROJECT_GRAPH_VERSION,
            name: name.into(),
            files: Vec::new(),
            packages: Vec::new(),
            entrypoints: Vec::new(),
            nodes: IndexMap::new(),
            edges: Vec::new(),
            build_system: None,
            test_framework: None,
        }
    }

    /// Count source files.
    pub fn source_file_count(&self) -> usize {
        self.files.iter().filter(|f| f.role == "source").count()
    }

    /// Count test files.
    pub fn test_file_count(&self) -> usize {
        self.files.iter().filter(|f| f.role == "test").count()
    }
}

/// Alias for docs.
pub type ProjectPackage = DependencyRef;
