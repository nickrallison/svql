pub use itertools;
pub use svql_common as common;
pub use svql_driver as driver;
pub use svql_subgraph as subgraph;
pub use tracing;

pub use crate::{
    Connection,
    // binding::PortResolver,
    // primitives::dff::*,
    // primitives::gates::*,
    report::ReportNode,
    // DataFrame API
    run_query,
};

// Trait re-exports
pub use crate::traits::{
    // Composite traits
    // CompositeComponent,
    // CompositeMatched,
    // ConnectionBuilder,
    // MatchedComponent,
    Pattern,
    // Component traits
    // SearchableComponent,
    // Topology,
    // // Variant traits
    // VariantComponent,
    // VariantMatched,
    // execute_netlist_query,
    // kind,
    // validate_composite,
    // validate_connection,
};

// New API types (Phase 1-6)
pub use crate::session::{
    // Core types
    ColumnDef,
    ColumnKind,
    // Execution
    // Config as ExecConfig,
    // ExecutionContext,
    // ExecutionPlan,
    // PatternRegistry,
    QueryError,
    Ref,
    // Row,
    // SearchRegistry,
    // Store,
    // Variants
    StoreVariantExt,
    Table,
    // TableBuilder,
    // // Trees
    // TempRef,
    // TreePostOrderIter,
    // TreePreOrderIter,
    // TreeRowBuilder,
    // TreeRowExt,
    // TreeTableBuilder,
    VariantIter,
    VariantPattern,
    VariantRef,
};

pub use common::{Config, ConfigBuilder, Dedupe, MatchLength, ModuleConfig, YosysModule};

pub use subgraph::{
    AssignmentSet, SingleAssignment,
    cell::{CellInfo, CellKind, CellWrapper, SourceLocation},
    graph_index::GraphIndex,
};

pub use driver::{Driver, DriverKey};

// pub use svql_macros::{composite, netlist, variant};
