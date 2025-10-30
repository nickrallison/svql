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
/// - For full usage (with imports like `tracing` and query types from `crate::queries`), see `svql_query/src/queries/enum_composites/and_any.rs`.
///
/// Limitations: Up to ~10 variants (tuple limits in parallel joins); no connections/validation.
pub fn enum_composite_inner(ts: TokenStream) -> TokenStream {
    let ast = parse::parse(ts.clone().into());
    let model = analyze::analyze(ast);
    let ir = lower::lower(model);
    codegen::codegen(ir).into()
}
