//! Semantic git blame / history transplant / cherry-pick stubs.

#![deny(unsafe_code)]
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SemanticBlame {
    pub target_location: String,
    pub source_location: Option<String>,
    pub commit: Option<String>,
    pub author: Option<String>,
    pub reason: String,
    pub supported: bool,
}

#[derive(Clone, Debug, Default)]
pub struct SemanticGit;

impl SemanticGit {
    pub fn blame(&self, location: &str) -> SemanticBlame {
        SemanticBlame {
            target_location: location.into(),
            source_location: None,
            commit: None,
            author: None,
            reason: "Semantic blame requires `.plxmap.json` + git history linkage (scaffold)".into(),
            supported: false,
        }
    }

    pub fn cherry_pick_status(&self, commit: &str) -> String {
        format!(
            "UNSUPPORTED: semantic cherry-pick of {commit} is scaffolded — needs PUIR semantic diff application"
        )
    }
}
