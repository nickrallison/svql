//! Primitive component traits for direct cell type matching.
//!
//! Unlike netlists which use subgraph matching against external files,
//! primitives match directly against cell types in the design index.

use crate::{
    CellId, Wire,
    prelude::*,
    session::{ColumnEntry, EntryArray, ExecutionContext, Port, QueryError, Row, Store, Table},
    traits::{Component, PatternInternal, kind, search_table_any},
};
use svql_driver::{Driver, DriverKey};
use tracing::debug;

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
    fn cell_filter(cell: &prjunnamed_netlist::Cell) -> bool {
        let _ = cell;
        true // Default: accept all cells of the matching kind
    }

    /// Schema accessor (macro generates this with OnceLock pattern).
    fn schema() -> &'static crate::session::PatternSchema {
        static SCHEMA: std::sync::OnceLock<crate::session::PatternSchema> =
            std::sync::OnceLock::new();
        SCHEMA.get_or_init(|| {
            let defs = Self::ports_to_defs();
            let defs_static: &'static [ColumnDef] = Box::leak(defs.into_boxed_slice());
            crate::session::PatternSchema::new(defs_static)
        })
    }

    /// Convert port declarations to column definitions.
    fn ports_to_defs() -> Vec<ColumnDef> {
        Self::PORTS
            .iter()
            .map(|p| ColumnDef::new(p.name, ColumnKind::Cell, false).with_direction(p.direction))
            .collect()
    }

    /// Resolve a cell wrapper into a row entry.
    ///
    /// For primitives, all ports typically point to the same underlying cell.
    fn resolve(cell_id: CellId) -> EntryArray {
        let schema_size = Self::PORTS.len();
        let entries: Vec<ColumnEntry> = (0..schema_size)
            .map(|_| ColumnEntry::Cell { id: Some(cell_id) })
            .collect();
        EntryArray::new(entries)
    }

    /// Rehydrate from row.
    fn rehydrate<'a>(
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

    fn schema() -> &'static crate::session::PatternSchema {
        T::schema()
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

        let haystack_container = ctx
            .driver()
            .get_design(&haystack_key, &ctx.config().haystack_options)
            .map_err(|e| QueryError::design_load(e.to_string()))?;

        let index = haystack_container.index();

        // Find all cells of the matching kind
        let mut row_matches = Vec::new();

        if let Some(cells) = index.cells_of_type_iter(T::CELL_KIND) {
            for cell_wrapper in cells {
                // Apply optional filter
                if T::cell_filter(cell_wrapper.get()) {
                    let cell_id = CellId::from_u64(cell_wrapper.debug_index() as u64);
                    row_matches.push(T::resolve(cell_id));
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

    fn rehydrate<'a>(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Primitive> + Send + Sync + 'static,
    {
        <T as Primitive>::rehydrate(row, store, driver, key)
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

        impl $crate::traits::Primitive for $name {
            const CELL_KIND: CellKind =
               CellKind::$cell_kind;

            const PORTS: &'static [$crate::session::Port] = &[
                $($crate::session::Port::$direction(stringify!($port))),*
            ];

            fn rehydrate<'a>(
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

        impl $crate::traits::Primitive for $name {
            const CELL_KIND: CellKind =
                CellKind::Dff;

            const PORTS: &'static [$crate::session::Port] = &[
                $($crate::session::Port::$direction(stringify!($port))),*
            ];

            fn cell_filter(cell: &prjunnamed_netlist::Cell) -> bool {
                let filter_fn: fn(&prjunnamed_netlist::Cell) -> bool = $filter;
                filter_fn(cell)
            }

            fn rehydrate<'a>(
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

#[cfg(test)]
mod tests {
    use super::*;
    use svql_common::Dedupe;
    use svql_query::query_test;

    // Example primitive gate
    define_primitive!(AndGate, And, [(a, input), (b, input), (y, output)]);

    query_test!(
        name: test_and2gates_small_and_tree_dedupe_none,
        query: AndGate,
        haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
        expect: 6,
        config: |config_builder| config_builder.dedupe(Dedupe::None)
    );

    query_test!(
        name: test_and2gates_small_and_tree_dedupe_all,
        query: AndGate,
        haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
        expect: 3,
        config: |config_builder| config_builder.dedupe(Dedupe::All)
    );
}
