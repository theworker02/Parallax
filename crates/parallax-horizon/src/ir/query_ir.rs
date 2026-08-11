//! QueryIR — ORM/query semantic reconstruction.

#![deny(unsafe_code)]
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryIr {
    pub statements: Vec<QueryStmt>,
    pub transactions: Vec<String>,
    pub notes: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryStmt {
    pub kind: String,
    pub table: Option<String>,
    pub filters: Vec<String>,
    pub joins: Vec<String>,
}

impl QueryIr {
    pub fn empty() -> Self {
        Self {
            statements: Vec::new(),
            transactions: Vec::new(),
            notes: "ORM chains lower into QueryIR before target SQL/ORM emission".into(),
        }
    }
}
