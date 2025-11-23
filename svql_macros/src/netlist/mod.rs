//! Macros for defining netlist queries.
//!
//! Netlist queries define structural patterns to match against a design's netlist,
//! specifying instances, ports, and connections.

use proc_macro::TokenStream;

pub mod analyze;
pub mod codegen;
pub mod lower;
pub mod parse;

/// Macro for defining netlist queries (structural patterns over instances/ports/connections).
/// - Generates: Struct `MyNetlist<S>`, `WithPath`, `Netlist`, `MatchedNetlist`, and `SearchableNetlist`
///   (with query execution, port mapping, connection validation).
/// - Instances/Ports: Define hierarchy and I/O.
/// - Connections: Validates linked ports (input/output matching).
/// - For full usage (with query types and traits like `NetlistMeta`), see `svql_query/src/queries/netlist/...`.
///
/// Limitations: Flat netlists only; no recursion (use composites for hierarchy).
pub fn netlist_inner(ts: TokenStream) -> TokenStream {
    let ast = parse::parse(ts.clone().into());
    let model = analyze::analyze(ast);
    let ir = lower::lower(model);
    codegen::codegen(ir).into()
}
