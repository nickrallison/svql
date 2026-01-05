pub use itertools;
pub use svql_common as common;
pub use svql_driver as driver;
pub use svql_subgraph as subgraph;
pub use tracing;

pub use crate::{
    Connection, Match, Search, State, Wire,
    binding::PortResolver,
    execute_query,
    instance::Instance,
    primitives::dff::*,
    primitives::gates::*,
    report::ReportNode,
    traits::validate_connection,
    traits::{
        Component, Projected, Query, Reportable, Searchable,
        composite::{ConnectionBuilder, Topology},
    },
};

pub use common::{Config, Dedupe, MatchLength, ModuleConfig, YosysModule};

pub use subgraph::{
    AssignmentSet, SingleAssignment,
    cell::{CellInfo, CellWrapper, SourceLocation},
    graph_index::GraphIndex,
};

pub use driver::{Driver, DriverKey, context::Context};

pub use svql_macros::{composite, netlist, variant};
