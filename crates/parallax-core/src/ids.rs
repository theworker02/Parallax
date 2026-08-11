//! Stable identifiers used across Parallax operations.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

macro_rules! id_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            /// Generate a new random identifier.
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            /// Borrow the inner UUID.
            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<Uuid> for $name {
            fn from(value: Uuid) -> Self {
                Self(value)
            }
        }
    };
}

id_type!(
    /// Identifies a single guest execution.
    ExecutionId
);
id_type!(
    /// Identifies a runtime worker instance.
    RuntimeId
);
id_type!(
    /// Identifies a migration operation.
    MigrationId
);
id_type!(
    /// Identifies a snapshot.
    SnapshotId
);
id_type!(
    /// Correlates protocol request/response pairs.
    RequestId
);

/// Stable object identity within a PIR graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(transparent)]
pub struct ObjectId(pub u64);

impl ObjectId {
    /// Create an object id from a raw value.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Raw numeric id.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique() {
        assert_ne!(ExecutionId::new(), ExecutionId::new());
    }

    #[test]
    fn object_id_display() {
        assert_eq!(ObjectId::new(41).to_string(), "#41");
    }
}
