//! Compact binary envelope for UES (magic + version + JSON body).

use crate::state::UniversalExecutionState;
use crate::version::UES_FORMAT_VERSION;
use crate::{from_json_bytes, to_json_bytes_compact};
use parallax_core::{ErrorCode, ParallaxError, Result};

/// Magic bytes: `PLXUES\0`.
pub const UES_MAGIC: &[u8] = b"PLXUES\0";

/// Encode UES as magic || u32 LE format_version || JSON bytes.
pub fn to_binary(ues: &UniversalExecutionState) -> Result<Vec<u8>> {
    ues.validate()?;
    let body = to_json_bytes_compact(ues)?;
    let mut out = Vec::with_capacity(UES_MAGIC.len() + 4 + body.len());
    out.extend_from_slice(UES_MAGIC);
    out.extend_from_slice(&ues.format_version.to_le_bytes());
    out.extend_from_slice(&body);
    Ok(out)
}

/// Decode binary UES envelope.
pub fn from_binary(bytes: &[u8]) -> Result<UniversalExecutionState> {
    if bytes.len() < UES_MAGIC.len() + 4 {
        return Err(ParallaxError::new(
            ErrorCode::InvalidSnapshot,
            "UES binary too short",
        )
        .with_source("parallax-ues")
        .with_operation("from_binary"));
    }
    if &bytes[..UES_MAGIC.len()] != UES_MAGIC {
        return Err(ParallaxError::new(
            ErrorCode::InvalidSnapshot,
            "invalid UES magic",
        )
        .with_source("parallax-ues")
        .with_operation("from_binary"));
    }
    let ver_off = UES_MAGIC.len();
    let version = u32::from_le_bytes([
        bytes[ver_off],
        bytes[ver_off + 1],
        bytes[ver_off + 2],
        bytes[ver_off + 3],
    ]);
    if version == 0 || version > UES_FORMAT_VERSION {
        return Err(ParallaxError::new(
            ErrorCode::InvalidSnapshot,
            format!(
                "unsupported UES format version {version} (supported: 1..={UES_FORMAT_VERSION})"
            ),
        )
        .with_source("parallax-ues")
        .with_operation("from_binary"));
    }
    from_json_bytes(&bytes[ver_off + 4..])
}
