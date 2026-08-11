//! UES format versioning (re-exported from parallax-core; independent of PIR / PCIR).

pub use parallax_core::{check_ues_format, UES_FORMAT_VERSION};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_current() {
        check_ues_format(UES_FORMAT_VERSION).unwrap();
    }

    #[test]
    fn rejects_bad() {
        assert!(check_ues_format(0).is_err());
        assert!(check_ues_format(UES_FORMAT_VERSION + 1).is_err());
    }
}
