use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

pub mod analyze;
pub mod codegen;
pub mod lower;
pub mod parse;

/// Procedural macro for defining netlist queries (single-module patterns with inputs/outputs).
///
/// Usage in a query file (e.g., src/queries/netlist/basic/and.rs):
/// ```rust
/// use crate::netlist::{netlist, NetlistMeta, SearchableNetlist};
///
/// netlist! {
///     name: AndGate,
///     module_name: "and_gate",
///     file: "examples/patterns/basic/and/verilog/and_gate.v",
///     inputs: [a, b],
///     outputs: [y]
/// }
/// ```
///
/// - Generates: Struct `AndGate<S>`, `new(path)`, `WithPath`, `NetlistMeta` (with ports), and `SearchableNetlist` (with `from_subgraph` binding inputs/outputs).
/// - Discovery: build.rs regexes detect the generated `impl SearchableNetlist`.
/// - Limitations: Single-bit ports only (multi-bit via future extensions); assumes string literals for module_name/file.
pub fn netlist_inner(ts: TokenStream) -> TokenStream {
    let ast = parse::parse(ts.clone().into());
    let model = analyze::analyze(ast);
    let ir = lower::lower(model);
    codegen::codegen(ir).into()
}
