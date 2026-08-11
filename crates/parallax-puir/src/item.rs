//! PUIR module-level items.

use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::types::PuirType;
use crate::{Effects, NodeId, SourceSpan};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Visibility.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    /// Public / exported.
    Public,
    /// Module private.
    Private,
}

/// Function parameter.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Parameter {
    /// Name.
    pub name: String,
    /// Type.
    pub ty: PuirType,
    /// Optional default (constant expr).
    pub default: Option<Expr>,
}

/// Function / method.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Function {
    /// Node id.
    pub id: NodeId,
    /// Name.
    pub name: String,
    /// Parameters.
    pub params: Vec<Parameter>,
    /// Return type.
    pub return_type: PuirType,
    /// Generic parameter names.
    pub generics: Vec<String>,
    /// Visibility.
    pub visibility: Visibility,
    /// Effects.
    pub effects: Effects,
    /// Body statements.
    pub body: Vec<Stmt>,
    /// Doc comment.
    pub doc: Option<String>,
    /// Source span.
    pub span: Option<SourceSpan>,
    /// Async.
    pub async_: bool,
}

/// Struct / class / interface field.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Field {
    /// Name.
    pub name: String,
    /// Type.
    pub ty: PuirType,
    /// Doc.
    pub doc: Option<String>,
}

/// Nominal type declaration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TypeDef {
    /// Node id.
    pub id: NodeId,
    /// Name.
    pub name: String,
    /// Kind: struct | interface | class | enum | type_alias
    #[serde(alias = "kind")]
    pub type_kind: String,
    /// Fields.
    pub fields: Vec<Field>,
    /// Methods (for classes).
    pub methods: Vec<Function>,
    /// Visibility.
    pub visibility: Visibility,
    /// Doc.
    pub doc: Option<String>,
    /// Span.
    pub span: Option<SourceSpan>,
}

/// Import.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Import {
    /// Module specifier (e.g. express, ./service).
    pub from: String,
    /// Imported names (empty = side-effect / default only).
    pub names: Vec<String>,
    /// Default import name.
    pub default: Option<String>,
    /// Span.
    pub span: Option<SourceSpan>,
}

/// Export.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Export {
    /// Exported name.
    pub name: String,
    /// Optional rename.
    pub as_name: Option<String>,
}

/// Top-level item.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PuirItem {
    /// Function.
    Function(Function),
    /// Type definition.
    Type(TypeDef),
    /// Constant.
    Const {
        /// Node id.
        id: NodeId,
        /// Name.
        name: String,
        /// Type.
        ty: PuirType,
        /// Value.
        value: Expr,
        /// Visibility.
        visibility: Visibility,
        /// Span.
        span: Option<SourceSpan>,
    },
    /// Unsupported item.
    Unsupported {
        /// Node id.
        id: NodeId,
        /// Original.
        original: String,
        /// Span.
        span: Option<SourceSpan>,
    },
}

/// A module (file-level unit).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Module {
    /// Stable module id (usually relative path without extension).
    pub id: String,
    /// Source path relative to project root.
    pub path: String,
    /// Imports.
    pub imports: Vec<Import>,
    /// Exports.
    pub exports: Vec<Export>,
    /// Items in source order.
    pub items: Vec<PuirItem>,
    /// File-level doc.
    pub doc: Option<String>,
    /// Language of origin.
    pub origin_language: String,
    /// Extra metadata (e.g. express routes discovered).
    pub metadata: IndexMap<String, serde_json::Value>,
}
