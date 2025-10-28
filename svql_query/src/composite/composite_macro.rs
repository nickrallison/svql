// svql_query/src/composite/composite_macro.rs
//
// Declarative macros for simplifying composite and enum_composite query definitions.
// These generate boilerplate impls for WithPath, Composite, SearchableComposite/EnumComposite, etc.
// Assumes sub-patterns (e.g., AndGate) implement SearchableNetlist.
// Supports parallel querying via #[cfg(feature = "parallel")] (add `use std::thread;` and `use tracing::{event, Level};` in usage files).

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
///
/// # Usage
/// ```ignore
/// composite! {
///     name: SdffeThenAnd,
///     subs: [
///         sdffe: Sdffe,
///         and_gate: AndGate
///     ],
///     connections: [
///         sdffe . q => and_gate . a,
///         sdffe . q => and_gate . b
///     ]
/// }
/// ```
///
/// This generates:
/// - A struct `SdffeThenAnd<S>` with a `path` field and sub-pattern fields
/// - A `new(path)` constructor
/// - `WithPath<S>` implementation
/// - `Composite<S>` implementation with the specified connections
/// - `MatchedComposite<'ctx>` implementation  
/// - `SearchableComposite` implementation with context merging and parallel/sequential query support
///
/// # Connections
/// All connections are grouped into a single validation set, meaning at least one connection
/// must be valid for the composite match to be valid. Empty connections (omit the field entirely)
/// will generate `vec![]` which passes validation trivially.
struct TBD;

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
struct TBD1;
