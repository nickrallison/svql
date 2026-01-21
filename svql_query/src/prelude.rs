pub use itertools;
pub use svql_common as common;
pub use svql_driver as driver;
pub use svql_subgraph as subgraph;
pub use tracing;

pub use crate::{
    Connection, Match, Search, State, Wire, binding::PortResolver, execute_query,
    execute_query_session, execute_query_session_direct, instance::Instance, primitives::dff::*,
    primitives::gates::*, report::ReportNode,
};

// Trait re-exports
pub use crate::traits::{
    // Composite traits
    CompositeComponent,
    CompositeMatched,
    ConnectionBuilder,
    Hardware,
    Matched,
    MatchedComponent,
    // Netlist traits
    NetlistComponent,
    NetlistMatched,
    Pattern,
    // Component traits
    SearchableComponent,
    Topology,
    // Variant traits
    VariantComponent,
    VariantMatched,
    execute_netlist_query,
    kind,
    validate_composite,
    validate_connection,
};

// Session re-exports (DataFrame-based result storage)
// Legacy types (deprecated, use new API instead)
#[allow(deprecated)]
pub use crate::session::{
    CellRow, Dehydrate, DehydratedResults, DehydratedRow, DesignFrame, ForeignKey,
    ForeignKeyTarget, MatchRef, MatchRow, QueryResults, QuerySchema, RecursiveFieldDesc, Rehydrate,
    RehydrateContext, RehydrateIter, ResultStore, SearchDehydrate, Session, SessionBuilder,
    SessionError, SessionRehydrateExt, SubmoduleFieldDesc, WireFieldDesc, WireRef,
};

// New API types (Phase 1-6)
pub use crate::session::{
    // Core types
    CellId,
    ColumnDef,
    ColumnKind,
    // Execution
    ExecutionContext,
    ExecutionPlan,
    PatternRegistry,
    QueryError,
    Ref,
    Row,
    Store,
    // Variants
    StoreVariantExt,
    Table,
    TableBuilder,
    // Trees
    TempRef,
    TreePostOrderIter,
    TreePreOrderIter,
    TreeRowBuilder,
    TreeRowExt,
    TreeTableBuilder,
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

pub use driver::{Driver, DriverKey, context::Context};

pub use svql_macros::{composite, netlist, variant};
