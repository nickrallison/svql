pub use itertools;
pub use svql_common as common;
pub use svql_driver as driver;
pub use svql_subgraph as subgraph;
pub use tracing;

pub use crate::run_query;

pub use crate::session::{
    Alias, AnyTable, ColumnDef, ColumnEntry, ColumnKind, EntryArray, ExecInfo, ExecutionContext,
    ExecutionPlan, MetaValue, PatternSchema, PortMap, QueryError, Ref, Row, RowIndex, SearchFn,
    Store, Submodule, Table, VariantIndex,
};

pub use crate::session::execution::TableSlot;

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

pub use common::*;

pub use subgraph::{AssignmentSet, SingleAssignment};

pub use driver::{Driver, DriverKey};

pub use svql_macros::{Composite, Netlist, Variant};
