//! Content hashing for PIR documents and snapshots.

use sha2::{Digest, Sha256};

/// SHA-256 hex digest of canonical compact JSON.
pub fn content_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}
