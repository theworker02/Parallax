//! Stable semantic node identities (not line-number based).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// Stable Parallax semantic ID: `plx:<kind>:<hash8>`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SemanticId(String);

impl SemanticId {
    /// Build from kind + qualified name + signature fingerprint.
    pub fn derive(kind: &str, qualified_name: &str, signature: &str) -> Self {
        let mut h = Sha256::new();
        h.update(kind.as_bytes());
        h.update(b"\0");
        h.update(qualified_name.as_bytes());
        h.update(b"\0");
        h.update(signature.as_bytes());
        let digest = h.finalize();
        let short = hex::encode(&digest[..4]);
        Self(format!("plx:{kind}:{short}"))
    }

    /// Raw string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SemanticId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_across_calls() {
        let a = SemanticId::derive("function", "service.getWeather", "city:string->Weather");
        let b = SemanticId::derive("function", "service.getWeather", "city:string->Weather");
        assert_eq!(a, b);
    }

    #[test]
    fn changes_with_signature() {
        let a = SemanticId::derive("function", "add", "a,b");
        let b = SemanticId::derive("function", "add", "a,b,c");
        assert_ne!(a, b);
    }
}
