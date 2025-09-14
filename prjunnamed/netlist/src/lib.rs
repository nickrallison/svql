//! This library provides the in-memory form of the Project Unnamed IR.
//!
//! A [`Design`] is represented as a sea of [`Cell`]s identified by a contiguous range of indices,
//! connected by [`Net`]s and [`Value`]s that refer back to cells by their index. This representation
//! is equally suited for bit-level and word-level netlists, including bit-level cells with multiple
//! outputs.

mod cell;
mod design;
mod io;
mod logic;
mod metadata;
mod param;
mod parse;
mod print;
mod rewrite;
mod target;
mod value;

mod isomorphic;
mod smt;

pub use cell::{
    AssignCell, Cell, DLatch, FlipFlop, Instance, IoBuffer, MatchCell, Memory, MemoryPortRelation,
    MemoryReadFlipFlop, MemoryReadPort, MemoryWritePort, TargetCell,
};
pub use design::{CellRef, Design, WithMetadataGuard};
pub use io::{IoNet, IoValue};
pub use logic::{Const, Trit};
pub use metadata::{MetaItem, MetaItemRef, MetaStringRef, SourcePosition};
pub use param::ParamValue;
pub use parse::{ParseError, parse};
pub use rewrite::{RewriteNetSource, RewriteResult, RewriteRuleset, Rewriter};
pub use target::{
    Target, TargetCellImportError, TargetCellPurity, TargetImportError, TargetInput, TargetIo,
    TargetOutput, TargetParam, TargetParamKind, TargetPrototype, create_target, register_target,
};
pub use value::{ControlNet, Net, Value, ValueRepr};

pub use isomorphic::{NotIsomorphic, isomorphic};
#[cfg(feature = "easy-smt")]
pub use smt::easy_smt::EasySmtEngine;
pub use smt::{SmtEngine, SmtResponse};
