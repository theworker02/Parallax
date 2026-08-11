//! Parallax Continuation IR (PCIR).
//!
//! A portable **control-flow** subset used at Continuum safepoints.
//! PCIR is not a full guest ISA — adapters lower only supported regions.
//!
//! Distinct from PIR: PIR models **values**; PCIR models **control**.

#![deny(unsafe_code)]
#![warn(missing_docs)]

mod ops;
mod program;
mod version;

pub use ops::{
    BinaryOperator, CompareOperator, Operand, PcirOp, UnaryOperator, ValueId,
};
pub use program::{PcirBlock, PcirFunction, PcirModule, PcirProgram};
pub use version::{check_pcir_schema, PCIR_SCHEMA_VERSION};

use parallax_core::{ErrorCode, ParallaxError};

/// Result alias.
pub type Result<T> = parallax_core::Result<T>;

/// Serialize a PCIR program to pretty JSON.
pub fn to_json_bytes(program: &PcirProgram) -> Result<Vec<u8>> {
    serde_json::to_vec_pretty(program).map_err(|e| {
        ParallaxError::new(ErrorCode::SerializationFailure, e.to_string())
            .with_source("parallax-pcir")
            .with_operation("to_json_bytes")
    })
}

/// Serialize a PCIR program to compact JSON.
pub fn to_json_bytes_compact(program: &PcirProgram) -> Result<Vec<u8>> {
    serde_json::to_vec(program).map_err(|e| {
        ParallaxError::new(ErrorCode::SerializationFailure, e.to_string())
            .with_source("parallax-pcir")
            .with_operation("to_json_bytes_compact")
    })
}

/// Deserialize and validate a PCIR program.
pub fn from_json_bytes(bytes: &[u8]) -> Result<PcirProgram> {
    let program: PcirProgram = serde_json::from_slice(bytes).map_err(|e| {
        ParallaxError::new(ErrorCode::SerializationFailure, e.to_string())
            .with_source("parallax-pcir")
            .with_operation("from_json_bytes")
    })?;
    program.validate()?;
    Ok(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::{BinaryOperator, Operand, PcirOp, ValueId};

    #[test]
    fn round_trip_straight_line() {
        let mut prog = PcirProgram::new();
        let mut func = PcirFunction::new("main");
        let mut block = PcirBlock::new("entry");
        block.ops.push(PcirOp::Constant {
            dest: ValueId(0),
            value: serde_json::json!({"t":"int","v":{"decimal":"1"}}),
        });
        block.ops.push(PcirOp::Constant {
            dest: ValueId(1),
            value: serde_json::json!({"t":"int","v":{"decimal":"2"}}),
        });
        block.ops.push(PcirOp::BinaryOp {
            dest: ValueId(2),
            op: BinaryOperator::Add,
            lhs: Operand::Value(ValueId(0)),
            rhs: Operand::Value(ValueId(1)),
        });
        block.ops.push(PcirOp::Return {
            value: Some(Operand::Value(ValueId(2))),
        });
        func.blocks.push(block);
        prog.functions.push(func);
        let bytes = to_json_bytes(&prog).unwrap();
        let back = from_json_bytes(&bytes).unwrap();
        assert_eq!(back.schema, PCIR_SCHEMA_VERSION);
        assert_eq!(back.functions[0].blocks[0].ops.len(), 4);
    }

    #[test]
    fn rejects_future_schema() {
        let raw = br#"{"schema":99,"functions":[],"metadata":{}}"#;
        let err = from_json_bytes(raw).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSnapshot);
        assert!(err.message.contains("unsupported PCIR schema"));
    }
}
