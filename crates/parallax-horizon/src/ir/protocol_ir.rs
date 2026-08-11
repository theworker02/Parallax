//! ProtocolIR — HTTP / RPC / event contracts.

#![deny(unsafe_code)]
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProtocolIr {
    pub routes: Vec<HttpRoute>,
    pub messages: Vec<MessageShape>,
    pub notes: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HttpRoute {
    pub method: String,
    pub path: String,
    pub request: Option<String>,
    pub response: Option<String>,
    pub status: Option<u16>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageShape {
    pub name: String,
    pub fields: Vec<String>,
}

impl ProtocolIr {
    pub fn empty() -> Self {
        Self {
            routes: Vec::new(),
            messages: Vec::new(),
            notes: "Populate from route extraction / traffic capture".into(),
        }
    }
}
