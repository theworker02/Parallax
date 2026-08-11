//! Semantic-loss classification for cross-runtime conversion.

use serde::{Deserialize, Serialize};
use std::fmt;

/// How much meaning is lost when converting a value between runtimes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticLoss {
    /// Bit-identical or language-neutral equivalence.
    None,
    /// Representation differs but semantics are preserved.
    Safe,
    /// Semantics may diverge depending on value contents.
    PotentiallyLossy,
    /// Known semantic corruption (e.g. integer precision).
    Lossy,
    /// Cannot be represented in the target runtime.
    Unsupported,
}

impl SemanticLoss {
    /// Whether migration should be rejected under the default policy.
    pub fn blocks_default_migration(self) -> bool {
        matches!(self, Self::Lossy | Self::Unsupported)
    }

    /// Whether migration requires an explicit lossy policy.
    pub fn requires_allow_lossy(self) -> bool {
        matches!(self, Self::Lossy)
    }

    /// Merge two classifications, keeping the worse outcome.
    pub fn worsen(self, other: Self) -> Self {
        self.max(other)
    }
}

impl fmt::Display for SemanticLoss {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "NONE"),
            Self::Safe => write!(f, "SAFE"),
            Self::PotentiallyLossy => write!(f, "POTENTIALLY_LOSSY"),
            Self::Lossy => write!(f, "LOSSY"),
            Self::Unsupported => write!(f, "UNSUPPORTED"),
        }
    }
}

/// Explicit conversion policy controlling lossy migrations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversionPolicy {
    /// Permit known-lossy conversions (emits prominent diagnostics).
    pub allow_lossy: bool,
    /// Permit potentially-lossy conversions without extra confirmation.
    pub allow_potentially_lossy: bool,
    /// Prefer BigInt over Number for integers outside JS safe integer range.
    pub prefer_bigint: bool,
    /// Reject unsupported values instead of encoding them as `Unsupported`.
    pub reject_unsupported: bool,
}

impl Default for ConversionPolicy {
    fn default() -> Self {
        Self {
            allow_lossy: false,
            allow_potentially_lossy: true,
            prefer_bigint: true,
            reject_unsupported: false,
        }
    }
}

impl ConversionPolicy {
    /// Strict policy: no lossy conversions.
    pub fn strict() -> Self {
        Self {
            allow_lossy: false,
            allow_potentially_lossy: false,
            prefer_bigint: true,
            reject_unsupported: true,
        }
    }

    /// Permissive policy for experimentation.
    pub fn permissive() -> Self {
        Self {
            allow_lossy: true,
            allow_potentially_lossy: true,
            prefer_bigint: true,
            reject_unsupported: false,
        }
    }

    /// Decide whether a given semantic-loss level is allowed.
    pub fn allows(&self, loss: SemanticLoss) -> bool {
        match loss {
            SemanticLoss::None | SemanticLoss::Safe => true,
            SemanticLoss::PotentiallyLossy => self.allow_potentially_lossy,
            SemanticLoss::Lossy => self.allow_lossy,
            SemanticLoss::Unsupported => !self.reject_unsupported,
        }
    }
}

/// Largest integer exactly representable in IEEE-754 binary64 (JS Number).
pub const JS_MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;
/// Smallest integer exactly representable in IEEE-754 binary64 (JS Number).
pub const JS_MIN_SAFE_INTEGER: i64 = -9_007_199_254_740_991;

/// Classify integer → JavaScript Number conversion.
pub fn integer_to_js_number_loss(value: i128) -> SemanticLoss {
    if value >= i128::from(JS_MIN_SAFE_INTEGER) && value <= i128::from(JS_MAX_SAFE_INTEGER) {
        SemanticLoss::None
    } else {
        SemanticLoss::Lossy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_integer_is_none() {
        assert_eq!(integer_to_js_number_loss(42), SemanticLoss::None);
    }

    #[test]
    fn unsafe_integer_is_lossy() {
        assert_eq!(
            integer_to_js_number_loss(9_007_199_254_740_993),
            SemanticLoss::Lossy
        );
    }

    #[test]
    fn default_policy_rejects_lossy() {
        assert!(!ConversionPolicy::default().allows(SemanticLoss::Lossy));
    }
}
