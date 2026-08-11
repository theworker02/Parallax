//! Universal stack frames (mixed-origin capable).

use indexmap::IndexMap;
use parallax_core::{ObjectId, RuntimeKind};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Stable frame identity within a UES document.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FrameId(pub String);

impl FrameId {
    /// Fresh random frame id.
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl Default for FrameId {
    fn default() -> Self {
        Self::new()
    }
}

/// Source location when known.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    /// File / module path.
    pub file: Option<String>,
    /// 1-based line.
    pub line: Option<u32>,
    /// 1-based column.
    pub column: Option<u32>,
}

/// One frame on a Continuum call stack.
///
/// Frames may declare different [`RuntimeKind`] values to model mixed-origin stacks
/// in the data model (resume across origins remains capability-gated).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UniversalFrame {
    /// Frame id.
    pub frame_id: FrameId,
    /// Origin runtime for this frame (may differ across the stack).
    pub runtime: RuntimeKind,
    /// Function name if known.
    pub function: Option<String>,
    /// Module / package path if known.
    pub module: Option<String>,
    /// Source location.
    pub source_location: Option<SourceLocation>,
    /// Instruction / bytecode / safepoint position.
    pub instruction_position: Option<String>,
    /// Argument bindings (name → PIR object or inline PIR JSON via heap).
    #[serde(default)]
    pub arguments: IndexMap<String, serde_json::Value>,
    /// Local bindings.
    #[serde(default)]
    pub locals: IndexMap<String, serde_json::Value>,
    /// Compiler temporaries when known.
    #[serde(default)]
    pub temporaries: IndexMap<String, serde_json::Value>,
    /// Where to resume after return (frame id or label).
    pub return_target: Option<String>,
    /// Exception handler target when known.
    pub exception_target: Option<String>,
    /// Optional locals object id in the heap graph.
    pub locals_root: Option<ObjectId>,
    /// Adapter-specific metadata.
    #[serde(default)]
    pub runtime_metadata: IndexMap<String, serde_json::Value>,
}

impl UniversalFrame {
    /// Build a checkpoint frame for an explicit safepoint.
    pub fn checkpoint(
        runtime: RuntimeKind,
        function: impl Into<String>,
        module: impl Into<String>,
        label: impl Into<String>,
        locals: IndexMap<String, serde_json::Value>,
    ) -> Self {
        let label = label.into();
        Self {
            frame_id: FrameId::new(),
            runtime,
            function: Some(function.into()),
            module: Some(module.into()),
            source_location: None,
            instruction_position: Some(format!("safepoint:{label}")),
            arguments: IndexMap::new(),
            locals,
            temporaries: IndexMap::new(),
            return_target: None,
            exception_target: None,
            locals_root: None,
            runtime_metadata: IndexMap::from([(
                "safepoint_kind".into(),
                serde_json::json!("explicit_checkpoint"),
            )]),
        }
    }
}
