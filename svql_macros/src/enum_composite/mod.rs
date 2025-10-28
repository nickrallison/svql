pub mod analyze;
pub mod codegen;
pub mod lower;
pub mod parse;

/// Macro for defining enum_composite queries (disjoint variants over sub-netlists).
///
/// Usage in a query file (e.g., src/queries/enum_composite/and_any.rs):
/// ```rust
/// #[cfg(feature = "parallel")]
/// use std::thread;
/// use tracing::{event, Level};
///
/// use crate::composite::{enum_composite, EnumComposite};  // Import macro and traits
/// use crate::queries::netlist::basic::and::{AndGate, AndMux, AndNor};
///
/// enum_composite! {
///     name: AndAny,
///     variants: [
///         Gate ( "and_gate" ) : AndGate,
///         Mux  ( "and_mux" ) : AndMux,
///         Nor  ( "and_nor" ) : AndNor
///     ]
/// }
/// ```
///
/// - Generates: Enum `AndAny<S>` with variants, `WithPath` (delegates to inner), `EnumComposite`,
///   `MatchedEnumComposite`, and `SearchableEnumComposite` (merged context, chained queries).
/// - Parallel: Conditionally spawns threads per variant if "parallel" feature enabled.
/// - Variants: Each needs a literal instance name (e.g., `"and_gate"`) for `path.child(inst_name)`.
/// - Query: Runs sub-queries (parallel or sequential), maps to enum variants, chains results (no validation/connections).
/// - Discovery: build.rs regexes detect the generated `impl SearchableEnumComposite`.
pub struct TBD;
