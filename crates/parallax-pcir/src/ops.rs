//! Portable PCIR operations.

use serde::{Deserialize, Serialize};

/// SSA-style temporary / register id within a function.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ValueId(pub u32);

/// Operand: a value id or an immediate JSON-encoded PIR leaf.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "v", rename_all = "snake_case")]
pub enum Operand {
    /// Reference a prior SSA value.
    Value(ValueId),
    /// Immediate constant encoded as PIR-tagged JSON.
    Imm(serde_json::Value),
}

/// Binary operators in the portable subset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BinaryOperator {
    /// Addition.
    Add,
    /// Subtraction.
    Sub,
    /// Multiplication.
    Mul,
    /// Division.
    Div,
    /// Modulo.
    Mod,
    /// Bitwise and.
    BitAnd,
    /// Bitwise or.
    BitOr,
    /// Bitwise xor.
    BitXor,
}

/// Unary operators.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnaryOperator {
    /// Negation.
    Neg,
    /// Logical not.
    Not,
    /// Bitwise not.
    BitNot,
}

/// Comparison operators.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompareOperator {
    /// Equal.
    Eq,
    /// Not equal.
    Ne,
    /// Less than.
    Lt,
    /// Less or equal.
    Le,
    /// Greater than.
    Gt,
    /// Greater or equal.
    Ge,
}

/// Portable control / data ops. Adapters lower only supported regions into these.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "opcode", rename_all = "snake_case")]
pub enum PcirOp {
    /// Load from a named slot / binding.
    Load {
        /// Destination.
        dest: ValueId,
        /// Source name (local / global / heap slot).
        name: String,
    },
    /// Store into a named slot / binding.
    Store {
        /// Target name.
        name: String,
        /// Value.
        value: Operand,
    },
    /// Move / copy.
    Move {
        /// Destination.
        dest: ValueId,
        /// Source.
        src: Operand,
    },
    /// Materialize a constant (PIR-tagged JSON).
    Constant {
        /// Destination.
        dest: ValueId,
        /// Constant payload.
        value: serde_json::Value,
    },
    /// Binary arithmetic / bitwise.
    BinaryOp {
        /// Destination.
        dest: ValueId,
        /// Operator.
        op: BinaryOperator,
        /// Left.
        lhs: Operand,
        /// Right.
        rhs: Operand,
    },
    /// Unary.
    UnaryOp {
        /// Destination.
        dest: ValueId,
        /// Operator.
        op: UnaryOperator,
        /// Operand.
        arg: Operand,
    },
    /// Compare → boolean.
    Compare {
        /// Destination.
        dest: ValueId,
        /// Operator.
        op: CompareOperator,
        /// Left.
        lhs: Operand,
        /// Right.
        rhs: Operand,
    },
    /// Conditional branch.
    Branch {
        /// Condition.
        cond: Operand,
        /// True target block name.
        then_block: String,
        /// False target block name.
        else_block: String,
    },
    /// Unconditional jump.
    Jump {
        /// Target block.
        target: String,
    },
    /// Call a named function / intrinsic-like callee.
    Call {
        /// Optional result.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        dest: Option<ValueId>,
        /// Callee name.
        callee: String,
        /// Arguments.
        args: Vec<Operand>,
    },
    /// Return from current function.
    Return {
        /// Optional return value.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<Operand>,
    },
    /// Construct a composite (list/map/tuple) — shape in `kind`.
    Construct {
        /// Destination.
        dest: ValueId,
        /// Shape: `list`, `map`, `tuple`, `set`, …
        kind: String,
        /// Elements / entries.
        elements: Vec<Operand>,
    },
    /// Index into a collection.
    Index {
        /// Destination.
        dest: ValueId,
        /// Base collection.
        base: Operand,
        /// Index / key.
        index: Operand,
    },
    /// Begin / step an iterator (portable loop helper).
    Iterate {
        /// Destination (iterator or next element).
        dest: ValueId,
        /// Iterable.
        iterable: Operand,
        /// Phase: `begin` or `next`.
        phase: String,
    },
    /// Throw an exception value.
    Throw {
        /// Exception payload.
        value: Operand,
    },
    /// Catch region marker (pairs with a handler block).
    Catch {
        /// Handler block.
        handler: String,
        /// Exception binding name in handler.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        binding: Option<String>,
    },
    /// Await an async value (capability-gated).
    Await {
        /// Destination.
        dest: ValueId,
        /// Awaitable.
        value: Operand,
    },
    /// Yield from a generator / coroutine (capability-gated).
    Yield {
        /// Yielded value.
        value: Operand,
    },
    /// Host / adapter intrinsic (e.g. `parallax.checkpoint`).
    Intrinsic {
        /// Optional result.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        dest: Option<ValueId>,
        /// Intrinsic name.
        name: String,
        /// Arguments.
        args: Vec<Operand>,
    },
}
