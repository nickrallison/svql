//! Macros for defining composite queries.
//!
//! Composite queries allow combining multiple sub-queries (instances) into a larger
//! query structure, supporting hierarchical pattern matching.

use proc_macro::TokenStream;

pub mod analyze;
pub mod codegen;
pub mod lower;
pub mod parse;

/// Macro for defining composites queries (disjoint sub-queries over netlist instances).
/// - Generates: Struct `MyComposite<S>`, `WithPath` (delegates to inner), `Composite`,
///   `MatchedComposite`, and `SearchableComposite` (merged context, parallel/sequential joins).
/// - Parallel: Supports `#[cfg(feature = "parallel")]` for threaded execution (add to your query crate).
/// - Instances: Each is `("inst_name", Type)` for `path.child(inst_name)`.
/// - Connections: Optional; validates linked ports (e.g., output to input).
/// - For full usage (with imports like `tracing` and query types), see `svql_query/src/queries/composites/...`.
///
/// Limitations: Up to ~10 instances (tuple limits); requires `svql_query` for traits/integration.
pub fn composite_inner(ts: TokenStream) -> TokenStream {
    let ast = parse::parse(ts.clone().into());
    let model = analyze::analyze(ast);
    let ir = lower::lower(model);
    codegen::codegen(ir).into()
}
