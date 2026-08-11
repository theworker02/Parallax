//! Language-independent semantic patches (.plxp).

#![deny(unsafe_code)]
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

pub const PLXP_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SemanticPatch {
    pub format_version: u32,
    pub id: String,
    pub description: String,
    pub operations: Vec<PatchOp>,
    pub preconditions: Vec<String>,
    pub postconditions: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatchOp {
    MakeOptional { node: String },
    AddValidation { node: String, rule: String },
    ChangeStatus { from: u16, to: u16 },
    Note { text: String },
}

impl SemanticPatch {
    pub fn example() -> Self {
        Self {
            format_version: PLXP_FORMAT_VERSION,
            id: "example.email-optional".into(),
            description: "User.email becomes optional; validate before persistence".into(),
            operations: vec![
                PatchOp::MakeOptional {
                    node: "User.email".into(),
                },
                PatchOp::AddValidation {
                    node: "User".into(),
                    rule: "validate before persistence".into(),
                },
                PatchOp::ChangeStatus { from: 400, to: 422 },
            ],
            preconditions: vec!["User entity exists".into()],
            postconditions: vec!["email Option<_>; 422 on invalid".into()],
        }
    }
}
