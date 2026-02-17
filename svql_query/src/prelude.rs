//! Common traits and types for the svql_query crate.

pub use itertools;
pub use svql_common as common;
pub use svql_driver as driver;
pub use svql_subgraph as subgraph;
pub use tracing;

pub use crate::run_query;

pub use crate::session::{
    Alias, AnyTable, ColumnDef, ColumnEntry, ColumnKind, EntryArray, ExecInfo, ExecutionContext,
    ExecutionPlan, PatternSchema, PortDecl, PortMap, QueryError, Ref, Row, SearchFn, Store,
    Submodule, Table,
};

pub use crate::session::execution::TableSlot;

// Trait re-exports
pub use crate::dsl::traits::{
    Component, Pattern, PatternInternal,
    composite::Composite,
    composite::Connection,
    composite::Connections,
    display::{render_wire, render_wire_compact},
    kind,
    netlist::Netlist,
    primitive::Primitive,
    recursive::Recursive,
    search_table_any,
    variant::Variant,
};

pub use crate::selector::Selector;

pub use common::*;

pub use subgraph::{AssignmentSet, SingleAssignment};

pub use driver::{Driver, DriverKey};

// Re-export derive macros with same name (they don't conflict with traits in derives)
pub use svql_macros::{Composite, Netlist, Variant};
