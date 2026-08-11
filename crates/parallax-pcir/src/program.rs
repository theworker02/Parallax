//! PCIR program / function / block containers.

use crate::ops::PcirOp;
use crate::version::{check_pcir_schema, PCIR_SCHEMA_VERSION};
use indexmap::IndexMap;
use parallax_core::{ErrorCode, ParallaxError, Result};
use serde::{Deserialize, Serialize};

/// A basic block of PCIR ops.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PcirBlock {
    /// Block name (unique within function).
    pub name: String,
    /// Ordered operations.
    pub ops: Vec<PcirOp>,
}

impl PcirBlock {
    /// Create an empty named block.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ops: Vec::new(),
        }
    }
}

/// A PCIR function.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PcirFunction {
    /// Function name.
    pub name: String,
    /// Parameter names (bound to ValueIds 0..n-1 by convention).
    #[serde(default)]
    pub params: Vec<String>,
    /// Basic blocks.
    pub blocks: Vec<PcirBlock>,
    /// Free-form metadata.
    #[serde(default)]
    pub metadata: IndexMap<String, serde_json::Value>,
}

impl PcirFunction {
    /// Create an empty function shell.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            params: Vec::new(),
            blocks: Vec::new(),
            metadata: IndexMap::new(),
        }
    }
}

/// Optional module grouping (for mixed-origin stacks).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PcirModule {
    /// Module name / path.
    pub name: String,
    /// Functions in this module.
    pub functions: Vec<String>,
}

/// Top-level PCIR program.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PcirProgram {
    /// Schema version (independent of PIR).
    pub schema: u32,
    /// Functions.
    pub functions: Vec<PcirFunction>,
    /// Optional modules.
    #[serde(default)]
    pub modules: Vec<PcirModule>,
    /// Entry function name when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry: Option<String>,
    /// Free-form metadata.
    #[serde(default)]
    pub metadata: IndexMap<String, serde_json::Value>,
}

impl PcirProgram {
    /// Empty program at the current schema.
    pub fn new() -> Self {
        Self {
            schema: PCIR_SCHEMA_VERSION,
            functions: Vec::new(),
            modules: Vec::new(),
            entry: None,
            metadata: IndexMap::new(),
        }
    }

    /// Validate schema and basic structural invariants.
    pub fn validate(&self) -> Result<()> {
        check_pcir_schema(self.schema)?;
        for func in &self.functions {
            if func.name.is_empty() {
                return Err(ParallaxError::new(
                    ErrorCode::InvalidSnapshot,
                    "PCIR function with empty name",
                )
                .with_source("parallax-pcir")
                .with_operation("validate"));
            }
            let mut names = std::collections::HashSet::new();
            for block in &func.blocks {
                if !names.insert(block.name.clone()) {
                    return Err(ParallaxError::new(
                        ErrorCode::InvalidSnapshot,
                        format!(
                            "duplicate PCIR block name '{}' in function '{}'",
                            block.name, func.name
                        ),
                    )
                    .with_source("parallax-pcir")
                    .with_operation("validate"));
                }
            }
        }
        Ok(())
    }

    /// Build a minimal stub program representing an explicit checkpoint boundary.
    pub fn checkpoint_stub(label: &str) -> Self {
        let mut prog = Self::new();
        let mut func = PcirFunction::new("__parallax_checkpoint_region");
        let mut block = PcirBlock::new("entry");
        block.ops.push(PcirOp::Intrinsic {
            dest: None,
            name: "parallax.checkpoint".into(),
            args: vec![crate::ops::Operand::Imm(serde_json::json!({
                "t": "string",
                "v": label
            }))],
        });
        func.blocks.push(block);
        prog.functions.push(func);
        prog.entry = Some("__parallax_checkpoint_region".into());
        prog.metadata
            .insert("kind".into(), serde_json::json!("checkpoint_stub"));
        prog.metadata
            .insert("label".into(), serde_json::json!(label));
        prog
    }
}

impl Default for PcirProgram {
    fn default() -> Self {
        Self::new()
    }
}
