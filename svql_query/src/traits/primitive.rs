//! Primitive component traits for direct cell type matching.
//!
//! Unlike netlists which use subgraph matching against external files,
//! primitives match directly against cell types in the design index.

use crate::{
    CellId,
    prelude::*,
    session::{ColumnEntry, EntryArray, ExecutionContext, Port, QueryError, Row, Store, Table},
    traits::{Component, PatternInternal, kind, search_table_any},
};
use svql_driver::{Driver, DriverKey};
use tracing::debug;

/// Helper to extract cell ID from a Value
fn value_to_cell_id(value: &prjunnamed_netlist::Value) -> Option<CellId> {
    match value.as_net() {
        Some(net) => net.as_cell_index().map(|idx| CellId::new(idx as u32)).ok(),
        None => None,
    }
}

/// Helper to extract cell ID from a Net
fn net_to_cell_id(net: &prjunnamed_netlist::Net) -> Option<CellId> {
    net.as_cell_index().map(|idx| CellId::new(idx as u32)).ok()
}

/// Helper to extract cell ID from a `ControlNet` (for AIG gates)
fn control_net_to_cell_id(cnet: &prjunnamed_netlist::ControlNet) -> Option<CellId> {
    net_to_cell_id(&cnet.net())
}

/// Resolve primitive ports by extracting inputs from the cell structure.
///
/// For most gates, inputs are positional arguments and outputs are the cell itself.
/// This handles common gate patterns (And, Or, Xor, Not, etc.) automatically.
/// Resolve primitive ports by extracting inputs from the cell structure.
///
/// For gates, inputs come from their arguments and outputs are the cell itself.
/// This handles all gate patterns automatically.
pub fn resolve_primitive_ports(
    cell: &prjunnamed_netlist::Cell,
    cell_wrapper: &CellWrapper,
    schema: &PatternSchema,
) -> EntryArray {
    use prjunnamed_netlist::Cell;

    let cell_id = CellId::from_usize(cell_wrapper.debug_index());
    let schema_size = schema.defs.len();
    let mut entries = vec![ColumnEntry::Cell { id: None }; schema_size];

    // Extract input values based on cell type
    let input_values: Vec<Option<CellId>> = match cell {
        // 2-input gates: And, Or, Xor, Eq, Comparisons, etc.
        Cell::And(a, b)
        | Cell::Or(a, b)
        | Cell::Xor(a, b)
        | Cell::Eq(a, b)
        | Cell::ULt(a, b)
        | Cell::SLt(a, b)
        | Cell::Mul(a, b)
        | Cell::UDiv(a, b)
        | Cell::UMod(a, b)
        | Cell::SDivTrunc(a, b)
        | Cell::SDivFloor(a, b)
        | Cell::SModTrunc(a, b)
        | Cell::SModFloor(a, b) => {
            vec![value_to_cell_id(a), value_to_cell_id(b)]
        }

        // 1-input gates: Not, Buf
        Cell::Not(a) | Cell::Buf(a) => {
            vec![value_to_cell_id(a)]
        }

        // Mux: (sel, a, b) - note the Net for selector
        Cell::Mux(s, a, b) => {
            vec![net_to_cell_id(s), value_to_cell_id(a), value_to_cell_id(b)]
        }

        // Adc: (a, b, carry_in)
        Cell::Adc(a, b, ci) => {
            vec![value_to_cell_id(a), value_to_cell_id(b), net_to_cell_id(ci)]
        }

        // Aig: (a, b) - both are ControlNets
        Cell::Aig(a, b) => {
            vec![control_net_to_cell_id(a), control_net_to_cell_id(b)]
        }

        // Shift operations: (value, shift_amount, multiplier)
        Cell::Shl(a, b, _) | Cell::UShr(a, b, _) | Cell::SShr(a, b, _) | Cell::XShr(a, b, _) => {
            vec![value_to_cell_id(a), value_to_cell_id(b)]
        }

        // For complex cells (DFF, Memory, etc.) or I/O cells,
        // fall back to setting all ports to the cell itself
        _ => {
            for item in entries.iter_mut().take(schema_size) {
                *item = ColumnEntry::Cell { id: Some(cell_id) };
            }
            return EntryArray::new(entries);
        }
    };

    // Map extracted input values to schema ports based on direction
    let mut input_idx = 0;

    for (col_idx, col_def) in schema.defs.iter().enumerate() {
        match col_def.direction {
            PortDirection::Input => {
                // Assign the next input value in order
                if input_idx < input_values.len() {
                    entries[col_idx] = ColumnEntry::Cell {
                        id: input_values[input_idx],
                    };
                    input_idx += 1;
                } else {
                    // Shouldn't happen if schema matches cell structure
                    tracing::warn!(
                        "Schema has more input ports than cell provides for {:?}",
                        cell
                    );
                    entries[col_idx] = ColumnEntry::Cell { id: None };
                }
            }
            PortDirection::Output => {
                // Output is always the gate cell itself
                entries[col_idx] = ColumnEntry::Cell { id: Some(cell_id) };
            }
            PortDirection::Inout | PortDirection::None => {
                // For bidirectional or unspecified, use cell_id as fallback
                entries[col_idx] = ColumnEntry::Cell { id: Some(cell_id) };
            }
        }
    }

    // Validation: warn if we didn't use all extracted inputs
    if input_idx < input_values.len() {
        tracing::warn!(
            "Cell {:?} provided {} inputs but schema only consumed {}",
            cell,
            input_values.len(),
            input_idx
        );
    }

    EntryArray::new(entries)
}

/// Trait for primitive hardware components that match against cell types.
///
/// Primitives differ from `Netlist` components in that they match directly
/// against cell types in the design index rather than using subgraph matching.
/// This is more efficient for simple gates and flip-flops.
pub trait Primitive: Sized + Component<Kind = kind::Primitive> + Send + Sync + 'static {
    /// The cell kind this primitive matches against.
    const CELL_KIND: CellKind;

    /// Port declarations (macro-generated).
    const PORTS: &'static [Port];

    /// Optional filter function for specialized matching (e.g., flip-flops).
    ///
    /// If provided, only cells that pass this filter will be included.
    /// The filter receives a reference to the matched cell wrapper.
    #[must_use]
    fn cell_filter(cell: &prjunnamed_netlist::Cell) -> bool {
        let _ = cell;
        true // Default: accept all cells of the matching kind
    }

    /// Schema accessor (macro generates this with `OnceLock` pattern).
    fn primitive_schema() -> &'static crate::session::PatternSchema {
        static SCHEMA: std::sync::OnceLock<crate::session::PatternSchema> =
            std::sync::OnceLock::new();
        SCHEMA.get_or_init(|| {
            let defs = Self::ports_to_defs();
            let defs_static: &'static [ColumnDef] = Box::leak(defs.into_boxed_slice());
            crate::session::PatternSchema::new(defs_static)
        })
    }

    /// Convert port declarations to column definitions.
    #[must_use]
    fn ports_to_defs() -> Vec<ColumnDef> {
        Self::PORTS
            .iter()
            .map(|p| ColumnDef::new(p.name, ColumnKind::Cell, false).with_direction(p.direction))
            .collect()
    }

    /// Resolve a cell wrapper into a row entry.
    ///
    /// Default implementation handles common gate patterns automatically.
    /// Override this for special cases (e.g., DFFs with named ports).
    #[must_use]
    fn resolve(cell_wrapper: &CellWrapper<'_>) -> EntryArray {
        resolve_primitive_ports(cell_wrapper.get(), cell_wrapper, Self::primitive_schema())
    }

    /// Rehydrate from row.
    fn primitive_rehydrate(
        _row: &Row<Self>,
        _store: &Store,
        _driver: &Driver,
        _key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Primitive> + Send + Sync + 'static;
}

impl<T> PatternInternal<kind::Primitive> for T
where
    T: Primitive + Component<Kind = kind::Primitive> + Send + Sync + 'static,
{
    const DEFS: &'static [ColumnDef] = &[]; // Placeholder, not used

    const SCHEMA_SIZE: usize = T::PORTS.len();

    const EXEC_INFO: &'static crate::session::ExecInfo = &crate::session::ExecInfo {
        type_id: std::any::TypeId::of::<T>(),
        type_name: std::any::type_name::<T>(),
        search_function: |ctx| {
            search_table_any::<T>(ctx, <T as PatternInternal<kind::Primitive>>::search_table)
        },
        nested_dependancies: &[],
    };

    fn internal_schema() -> &'static crate::session::PatternSchema {
        T::primitive_schema()
    }

    fn preload_driver(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        // Primitives only need the haystack design loaded
        driver.preload_design(design_key, &config.haystack_options)?;
        Ok(())
    }

    fn search_table(ctx: &ExecutionContext) -> Result<Table<Self>, QueryError>
    where
        Self: Send + Sync + 'static,
    {
        let haystack_key = ctx.design_key();

        debug!(
            "Searching for primitive {} matching cell kind {:?}",
            std::any::type_name::<T>(),
            T::CELL_KIND
        );

        // Warn if Dedupe::None is used with primitives
        if matches!(ctx.config().dedupe, svql_common::Dedupe::None) {
            tracing::warn!(
                "Dedupe::None has no effect on primitive {} - primitives enumerate cells directly without combinatorial expansion. Use Dedupe::All instead, behavior does not change, but will match the expected behavior of netlists.",
                std::any::type_name::<T>()
            );
        }

        let haystack_container = ctx
            .driver()
            .get_design(&haystack_key, &ctx.config().haystack_options)
            .map_err(|e| QueryError::design_load(e.to_string()))?;

        let index = haystack_container.index();

        // Find all cells of the matching kind
        let mut row_matches = Vec::new();

        if let Some(cells) = index.cells_of_type_iter(T::CELL_KIND) {
            for cell_wrapper in cells {
                if T::cell_filter(cell_wrapper.get()) {
                    row_matches.push(T::resolve(cell_wrapper));
                }
            }
        }

        debug!(
            "Found {} matches for primitive {}",
            row_matches.len(),
            std::any::type_name::<T>()
        );

        Table::<Self>::new(row_matches)
    }

    fn internal_rehydrate<'a>(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Primitive> + Send + Sync + 'static,
    {
        Self::primitive_rehydrate(row, store, driver, key)
    }
}

/// Macro to define a primitive gate component.
///
/// # Example
/// ```ignore
/// define_primitive!(AndGate, And, [
///     (a, Input),
///     (b, Input),
///     (y, Output)
/// ]);
/// ```
#[macro_export]
macro_rules! define_primitive {
    (
        $name:ident,
        $cell_kind:ident,
        [$(($port:ident, $direction:ident)),* $(,)?]
    ) => {
        #[doc = concat!("A primitive ", stringify!($cell_kind), " gate component.")]
        #[derive(Debug, Clone)]
        pub struct $name {
            $(
                #[doc = concat!("The ", stringify!($port), " port wire.")]
                pub $port: $crate::Wire,
            )*
        }

        impl $crate::prelude::Primitive for $name {
            const CELL_KIND: CellKind =
               CellKind::$cell_kind;

            const PORTS: &'static [$crate::session::Port] = &[
                $($crate::session::Port::$direction(stringify!($port))),*
            ];

            fn primitive_schema() -> &'static $crate::session::PatternSchema {
                static SCHEMA: std::sync::OnceLock<$crate::session::PatternSchema> =
                    std::sync::OnceLock::new();
                SCHEMA.get_or_init(|| {
                    let defs = Self::ports_to_defs();
                    let defs_static: &'static [$crate::session::ColumnDef] =
                        Box::leak(defs.into_boxed_slice());
                    $crate::session::PatternSchema::new(defs_static)
                })
            }

            fn primitive_rehydrate<'a>(
                row: &$crate::session::Row<Self>,
                _store: &$crate::session::Store,
                _driver: &$crate::driver::Driver,
                _key: &$crate::driver::DriverKey,
            ) -> Option<Self>
            where
                Self: $crate::traits::Component +
                      $crate::traits::PatternInternal<$crate::traits::kind::Primitive> +
                      Send + Sync + 'static,
            {
                Some($name {
                    $($port: row.wire(stringify!($port))?),*
                })

            }
        }

        impl $crate::traits::Component for $name {
            type Kind = $crate::traits::kind::Primitive;
        }
    };
}

/// Macro to define a DFF primitive with custom filtering.
///
/// # Example
/// ```ignore
/// define_dff_primitive!(
///     Sdffe,
///     [
///         (clk, Input),
///         (d, Input),
///         (reset, Input),
///         (en, Input),
///         (q, Output)
///     ],
///     |cell| {
///         if let prjunnamed_netlist::Cell::Dff(ff) = cell {
///             ff.has_reset() && ff.has_enable()
///         } else {
///             false
///         }
///     }
/// );
/// ```
#[macro_export]
macro_rules! define_dff_primitive {
    (
        $name:ident,
        [$(($port:ident, $direction:ident)),* $(,)?],
        $filter:expr
    ) => {
        #[doc = concat!("A flip-flop primitive: ", stringify!($name))]
        #[derive(Debug, Clone)]
        pub struct $name {
            $(
                #[doc = concat!("The ", stringify!($port), " port wire.")]
                pub $port: $crate::Wire,
            )*
        }

        impl $crate::prelude::Primitive for $name {
            const CELL_KIND: CellKind =
                CellKind::Dff;

            const PORTS: &'static [$crate::session::Port] = &[
                $($crate::session::Port::$direction(stringify!($port))),*
            ];

            fn cell_filter(cell: &prjunnamed_netlist::Cell) -> bool {
                let filter_fn: fn(&prjunnamed_netlist::Cell) -> bool = $filter;
                filter_fn(cell)
            }

            fn primitive_schema() -> &'static $crate::session::PatternSchema {
                static SCHEMA: std::sync::OnceLock<$crate::session::PatternSchema> =
                    std::sync::OnceLock::new();
                SCHEMA.get_or_init(|| {
                    let defs = Self::ports_to_defs();
                    let defs_static: &'static [$crate::session::ColumnDef] =
                        Box::leak(defs.into_boxed_slice());
                    $crate::session::PatternSchema::new(defs_static)
                })
            }

            fn primitive_rehydrate<'a>(
                row: &$crate::session::Row<Self>,
                _store: &$crate::session::Store,
                _driver: &$crate::driver::Driver,
                _key: &$crate::driver::DriverKey,
            ) -> Option<Self>
            where
                Self: $crate::traits::Component +
                      $crate::traits::PatternInternal<$crate::traits::kind::Primitive> +
                      Send + Sync + 'static,
            {
                Some($name {
                    $(
                        $port: row.wire(stringify!($port))?,
                    )*
                })
            }
        }

        impl $crate::traits::Component for $name {
            type Kind = $crate::traits::kind::Primitive;
        }
    };
}

#[allow(dead_code)]
#[cfg(test)]
mod tests {
    use super::*;
    use svql_common::Dedupe;
    use svql_query::query_test;

    // Example primitive gate
    define_primitive!(TestAndGate, And, [(a, input), (b, input), (y, output)]);

    // small_and_tree has 3 AND gates total
    // Dedupe::None will warn but still return 3 matches (same as Dedupe::All)
    query_test!(
        name: test_and_gate_small_tree_dedupe_none,
        query: TestAndGate,
        haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
        expect: 3,  // Same as Dedupe::All - primitives don't have permutations
        config: |config_builder| config_builder.dedupe(Dedupe::None)
    );

    query_test!(
        name: test_and_gate_small_tree_dedupe_all,
        query: TestAndGate,
        haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
        expect: 3,
        config: |config_builder| config_builder.dedupe(Dedupe::All)
    );

    #[test]
    fn test_and_gate_rehydration() -> Result<(), Box<dyn std::error::Error>> {
        use crate::test_harness::setup_test_logging;

        setup_test_logging();

        let driver = Driver::new_workspace()?;
        let config = Config::builder().dedupe(Dedupe::All).build();
        let key = DriverKey::new(
            "examples/fixtures/basic/and/verilog/small_and_tree.v",
            "small_and_tree",
        );

        let store = svql_query::run_query::<TestAndGate>(&driver, &key, &config)?;
        let table = store.get::<TestAndGate>().expect("Table should exist");

        // Verify we can rehydrate and access fields
        let first_row = table.rows().next().expect("Should have at least one match");
        let gate = <TestAndGate as svql_query::traits::Pattern>::rehydrate(
            &first_row, &store, &driver, &key,
        )
        .expect("Should rehydrate");

        // Access fields to prove they work
        assert!(gate.a.id().raw() > 0, "Input A should have valid ID");
        assert!(gate.b.id().raw() > 0, "Input B should have valid ID");
        assert!(gate.y.id().raw() > 0, "Output Y should have valid ID");

        Ok(())
    }
}
