//! PUIR schema versioning (independent of PIR / UES / PCIR).

use parallax_core::{ErrorCode, ParallaxError, Remediation};

/// PUIR schema version. Bump on incompatible IR changes.
pub const PUIR_SCHEMA_VERSION: u32 = 1;

/// Reject unknown / future major schemas.
pub fn check_puir_schema(version: u32) -> Result<(), ParallaxError> {
    if version == 0 {
        return Err(ParallaxError::new(
            ErrorCode::InvalidArgument,
            "PUIR schema version 0 is invalid",
        )
        .with_source("parallax-puir")
        .with_operation("check_puir_schema"));
    }
    if version > PUIR_SCHEMA_VERSION {
        return Err(ParallaxError::new(
            ErrorCode::InvalidArgument,
            format!(
                "PUIR schema {version} is newer than supported {PUIR_SCHEMA_VERSION}"
            ),
        )
        .with_source("parallax-puir")
        .with_operation("check_puir_schema")
        .remediate(Remediation::new(
            "Upgrade Parallax to a release that supports this PUIR schema",
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_future() {
        assert!(check_puir_schema(PUIR_SCHEMA_VERSION + 1).is_err());
    }

    #[test]
    fn accepts_current() {
        assert!(check_puir_schema(PUIR_SCHEMA_VERSION).is_ok());
    }
}
