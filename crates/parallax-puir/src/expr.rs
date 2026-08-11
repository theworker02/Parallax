//! PUIR expressions — intent-preserving operations.

#![allow(missing_docs)]

use crate::types::PuirType;
use crate::{NodeId, SourceSpan};
use serde::{Deserialize, Serialize};

/// Expression node.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Expr {
    /// Literal constant.
    Constant {
        /// Node id.
        id: NodeId,
        /// Value as JSON (numbers, strings, bools, null).
        value: serde_json::Value,
        /// Optional span.
        span: Option<SourceSpan>,
    },
    /// Local / global name.
    Name {
        /// Node id.
        id: NodeId,
        /// Identifier.
        name: String,
        span: Option<SourceSpan>,
    },
    /// Assignment result / move.
    Assign {
        id: NodeId,
        /// Target name.
        target: String,
        /// Value.
        value: Box<Expr>,
        span: Option<SourceSpan>,
    },
    /// Function / method call.
    Call {
        id: NodeId,
        /// Callee expression.
        callee: Box<Expr>,
        /// Arguments.
        args: Vec<Expr>,
        span: Option<SourceSpan>,
    },
    /// Field access.
    AccessField {
        id: NodeId,
        object: Box<Expr>,
        field: String,
        span: Option<SourceSpan>,
    },
    /// Indexing.
    Index {
        id: NodeId,
        collection: Box<Expr>,
        index: Box<Expr>,
        span: Option<SourceSpan>,
    },
    /// Binary operator.
    BinaryOp {
        id: NodeId,
        /// Operator symbol (`+`, `-`, `===`, …).
        operator: String,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Option<SourceSpan>,
    },
    /// Unary operator.
    UnaryOp {
        id: NodeId,
        /// Operator symbol.
        operator: String,
        operand: Box<Expr>,
        span: Option<SourceSpan>,
    },
    /// Construct object / struct / class instance.
    Construct {
        id: NodeId,
        /// Type name if known.
        type_name: Option<String>,
        /// Fields.
        fields: Vec<(String, Expr)>,
        span: Option<SourceSpan>,
    },
    /// List literal.
    List {
        id: NodeId,
        elements: Vec<Expr>,
        span: Option<SourceSpan>,
    },
    /// Filter intent (e.g. list comprehension / Array.filter).
    Filter {
        id: NodeId,
        collection: Box<Expr>,
        /// Predicate parameter name.
        param: String,
        predicate: Box<Expr>,
        span: Option<SourceSpan>,
    },
    /// Map intent.
    Map {
        id: NodeId,
        collection: Box<Expr>,
        param: String,
        body: Box<Expr>,
        span: Option<SourceSpan>,
    },
    /// Await.
    Await {
        id: NodeId,
        value: Box<Expr>,
        span: Option<SourceSpan>,
    },
    /// Cast / convert.
    Convert {
        id: NodeId,
        value: Box<Expr>,
        to: PuirType,
        span: Option<SourceSpan>,
    },
    /// Intrinsic (JSON.parse, env.get, fs.read, http.get, …).
    Intrinsic {
        id: NodeId,
        /// Intrinsic name in Parallax vocabulary.
        name: String,
        args: Vec<Expr>,
        span: Option<SourceSpan>,
    },
    /// Unsupported expression preserved for review.
    Unsupported {
        id: NodeId,
        /// Original snippet or description.
        original: String,
        span: Option<SourceSpan>,
    },
}

impl Expr {
    /// Node id.
    pub fn id(&self) -> NodeId {
        match self {
            Self::Constant { id, .. }
            | Self::Name { id, .. }
            | Self::Assign { id, .. }
            | Self::Call { id, .. }
            | Self::AccessField { id, .. }
            | Self::Index { id, .. }
            | Self::BinaryOp { id, .. }
            | Self::UnaryOp { id, .. }
            | Self::Construct { id, .. }
            | Self::List { id, .. }
            | Self::Filter { id, .. }
            | Self::Map { id, .. }
            | Self::Await { id, .. }
            | Self::Convert { id, .. }
            | Self::Intrinsic { id, .. }
            | Self::Unsupported { id, .. } => *id,
        }
    }
}
