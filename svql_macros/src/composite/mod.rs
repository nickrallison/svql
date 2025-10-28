use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

pub mod analyze;
pub mod codegen;
pub mod lower;
pub mod parse;

/// Macro for defining composite queries (structural patterns with sub-netlists and required connections).
///
/// Usage in a query file (e.g., src/queries/composite/dff_then_and.rs):
/// ```rust
/// #[cfg(feature = "parallel")]
/// use std::thread;
/// use tracing::{event, Level};
///
/// use crate::composite::{composite, Composite};  // Import macro and traits
/// use crate::queries::netlist::basic::{dff::Sdffe, and::AndGate};
///
/// composite! {
///     name: SdffeThenAnd,
///     subs: [ sdffe: Sdffe, and_gate: AndGate ],
///     connections: [
///         [
///             sdffe . q => and_gate . a,
///             sdffe . q => and_gate . b
///         ]
///     ]
/// }
/// ```
///
/// - Generates: Struct `SdffeThenAnd<S>`, `new(path)`, `WithPath`, `Composite` (with connections in one group),
///   `MatchedComposite`, and `SearchableComposite` (with merged context and iproduct!-based query + validation).
/// - Parallel: Conditionally spawns threads per sub-query if "parallel" feature enabled.
/// - Connections: All in a single validation group (must have at least one valid connection).
/// - Query: Runs sub-queries (parallel or sequential), uses `iproduct!` to combine, filters via `validate_connections`.
/// - Empty connections: Allowed (uses `vec![vec![]]`—validation always passes).
/// - Limitations: Up to ~10 subs (due to `iproduct!` tuple limits); one connection group.
/// - Discovery: build.rs regexes detect the generated `impl SearchableComposite`.
/// Macro for defining composite queries (structural patterns with sub-netlists and required connections).
pub fn composite_inner(ts: TokenStream) -> TokenStream {
    let ast = parse::parse(ts.clone().into());
    let model = analyze::analyze(ast);
    let ir = lower::lower(model);
    let _ = codegen::codegen(ir);
    TokenStream::new()
}
