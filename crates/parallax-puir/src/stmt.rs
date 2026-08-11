//! PUIR statements.

#![allow(missing_docs)]

use crate::expr::Expr;
use crate::{NodeId, SourceSpan};
use serde::{Deserialize, Serialize};

/// Statement node.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Stmt {
    /// Local declaration.
    Declare {
        id: NodeId,
        name: String,
        mutable: bool,
        value: Option<Expr>,
        span: Option<SourceSpan>,
    },
    /// Expression statement.
    Expr {
        id: NodeId,
        expr: Expr,
        span: Option<SourceSpan>,
    },
    /// Return.
    Return {
        id: NodeId,
        value: Option<Expr>,
        span: Option<SourceSpan>,
    },
    /// Branch.
    Branch {
        id: NodeId,
        condition: Expr,
        then_body: Vec<Stmt>,
        else_body: Vec<Stmt>,
        span: Option<SourceSpan>,
    },
    /// Loop (while / for intent collapsed).
    Loop {
        id: NodeId,
        /// Optional iterator binding.
        binding: Option<String>,
        /// Collection or condition.
        header: Expr,
        body: Vec<Stmt>,
        /// `while` | `for_each` | `infinite`
        kind: String,
        span: Option<SourceSpan>,
    },
    /// Throw.
    Throw {
        id: NodeId,
        value: Expr,
        span: Option<SourceSpan>,
    },
    /// Try / catch.
    Catch {
        id: NodeId,
        try_body: Vec<Stmt>,
        catch_name: Option<String>,
        catch_body: Vec<Stmt>,
        span: Option<SourceSpan>,
    },
    /// Assign statement.
    Assign {
        id: NodeId,
        target: String,
        value: Expr,
        span: Option<SourceSpan>,
    },
    /// Unsupported statement.
    Unsupported {
        id: NodeId,
        original: String,
        span: Option<SourceSpan>,
    },
}
