pub use itertools;
pub use svql_common as common;
pub use svql_driver as driver;
pub use svql_subgraph as subgraph;
pub use tracing;

pub use crate::{
    // Core types
    CellId,
    Wire,
    // binding::PortResolver,
    // primitives::dff::*,
    // primitives::gates::*,
    // report::ReportNode,
    // DataFrame API
    run_query,
};

// Session re-exports
pub use crate::session::{
    // Core types
    Alias,
    AnyTable,
    ColumnDef,
    ColumnEntry,
    ColumnKind,
    ConnectionBuilder,
    EntryArray,
    // Execution
    ExecInfo,
    ExecutionContext,
    ExecutionPlan,
    PatternSchema,
    Port,
    PortDirection,
    PortMap,
    QueryError,
    Ref,
    Row,
    SearchFn,
    Store,
    Submodule,
    Table,
};

// Trait re-exports
pub use crate::traits::{
    Component, Pattern, PatternInternal, composite::Composite, composite::Connection,
    composite::Connections, kind, netlist::Netlist, primitive::Primitive, search_table_any,
    variant::Variant,
};

pub use crate::selector::Selector;

pub use common::{Config, ConfigBuilder, Dedupe, MatchLength, ModuleConfig, YosysModule};

pub use subgraph::{
    AssignmentSet, SingleAssignment,
    cell::{CellInfo, CellKind, CellWrapper, SourceLocation},
    graph_index::GraphIndex,
};

pub use driver::{Driver, DriverKey};

// Re-export derive macros with same name (they don't conflict with traits in derives)
pub use svql_macros::{Composite, Netlist, Variant};
