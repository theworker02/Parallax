//! Independently versioned PCIR schema (re-exported from parallax-core).

pub use parallax_core::{check_pcir_schema, PCIR_SCHEMA_VERSION};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_current() {
        check_pcir_schema(PCIR_SCHEMA_VERSION).unwrap();
    }

    #[test]
    fn rejects_zero_and_future() {
        assert!(check_pcir_schema(0).is_err());
        assert!(check_pcir_schema(PCIR_SCHEMA_VERSION + 1).is_err());
    }
}
