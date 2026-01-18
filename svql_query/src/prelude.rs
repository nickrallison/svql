pub use itertools;
pub use svql_common as common;
pub use svql_driver as driver;
pub use svql_subgraph as subgraph;
pub use tracing;

pub use crate::{
    Connection, Match, Search, State, Wire, binding::PortResolver, execute_query,
    execute_query_session, instance::Instance, primitives::dff::*, primitives::gates::*,
    report::ReportNode,
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
pub use crate::session::{
    CellRow, Dehydrate, DehydratedRow, DesignFrame, MatchRef, MatchRow, QueryResults, QuerySchema,
    RecursiveFieldDesc, Rehydrate, RehydrateContext, RehydrateIter, ResultStore, Session,
    SessionBuilder, SessionError, SessionRehydrateExt, SubmoduleFieldDesc, WireFieldDesc, WireRef,
};

pub use common::{Config, ConfigBuilder, Dedupe, MatchLength, ModuleConfig, YosysModule};

pub use subgraph::{
    AssignmentSet, SingleAssignment,
    cell::{CellInfo, CellKind, CellWrapper, SourceLocation},
    graph_index::GraphIndex,
};

pub use driver::{Driver, DriverKey, context::Context};

pub use svql_macros::{composite, netlist, variant};
