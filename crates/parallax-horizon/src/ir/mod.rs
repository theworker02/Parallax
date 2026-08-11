//! Concurrency, protocol, and query IR; shape inference; type crystallization.

mod concurrency_ir;
mod protocol_ir;
mod query_ir;
mod shape_inference;
mod type_crystallizer;

pub use concurrency_ir::{ConcurrencyGraph, ConcurrencyIntent};
pub use protocol_ir::{HttpRoute, MessageShape, ProtocolIr};
pub use query_ir::{QueryIr, QueryStmt};
pub use shape_inference::{FieldType, Shape, ShapeInferencer};
pub use type_crystallizer::{CrystallizedKind, CrystallizedType, TypeCrystallizer};
