//! Macros for defining enum composite queries.
//!
//! Enum composites allow defining a query that matches one of several possible
//! sub-queries (variants), enabling polymorphic pattern matching.

// svql_macros/src/enum_composite/mod.rs
use proc_macro::TokenStream;

pub mod analyze;
pub mod codegen;
pub mod lower;
pub mod parse;

/// Macro for defining enum_composites queries (disjoint variants over sub-netlists).
/// - Note: Commas required between variants (trailing optional).
/// - Generates: Enum `MyEnum<S>` with variants, `WithPath` (delegates to inner), `EnumComposite`,
///   `MatchedEnumComposite`, and `SearchableEnumComposite` (merged context, chained queries).
/// - Parallel: Supports `#[cfg(feature = "parallel")]` for threaded execution (add to your query crate).
/// - Variants: Each is `(VariantName, "inst_name", Type)` for `path.child(inst_name)`.
/// - Query: Runs sub-queries (parallel or sequential), maps to enum variants, chains results (no validation).
///
/// # NEW: Common Ports
/// Use `common_ports: { field: "method" }` to auto-generate accessors for shared ports:
/// ```rust,ignore
/// enum_composite! {
///     name: DffAny,
///     variants: [(Simple, "dff", SimpleDff), (Sync, "sdff", SyncDff)],
///     common_ports: {
///         clk: "clock",
///         d: "data_input",
///         q: "output"
///     }
/// }
/// // Generates: dff_any.clock() -> &Wire<S>, etc.
/// ```
///
/// For full usage, see `svql_query/src/queries/enum_composites/and_any.rs` or `dff_any.rs`.
///
/// Limitations: Up to ~10 variants (tuple limits in parallel joins); no connections/validation.
pub fn enum_composite_inner(ts: TokenStream) -> TokenStream {
    let ast = parse::parse(ts.clone().into());
    let model = analyze::analyze(ast);
    let ir = lower::lower(model);
    codegen::codegen(ir).into()
}
