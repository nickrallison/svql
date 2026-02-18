//! Primitive component traits for direct cell type matching.
//!
//! Unlike netlists which use subgraph matching against external files,
//! primitives match directly against cell types in the design index.

use crate::{
    prelude::*,
    session::{ColumnEntry, EntryArray, ExecutionContext, QueryError, Row, Store, Table},
    traits::{Component, PatternInternal, kind, search_table_any},
};
use svql_driver::{Driver, DriverKey};

/// Trait for primitive hardware components that match against cell types.
///
/// Primitives differ from `Netlist` components in that they match directly
/// against cell types in the design index rather than using subgraph matching.
/// This is more efficient for simple gates and flip-flops.
pub trait Primitive: Sized + Component<Kind = kind::Primitive> + Send + Sync + 'static {
    /// The cell kind this primitive matches against.
    const CELL_KIND: CellKind;

    /// Port declarations (macro-generated).
    const PORTS: &'static [PortDecl];

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
            .map(|p| ColumnDef::new(p.name, ColumnKind::Wire, false).with_direction(p.direction))
            .collect()
    }

    /// Resolve a cell wrapper into a row entry.
    ///
    /// Default implementation handles common gate patterns automatically.
    /// Override this for special cases (e.g., DFFs with named ports).
    #[must_use]
    fn resolve(wrapper: &CellWrapper<'_>) -> EntryArray {
        let schema = Self::primitive_schema();
        let mut entries = vec![ColumnEntry::Null; schema.defs.len()];

        for (idx, col_def) in schema.defs.iter().enumerate() {
            let entry = match col_def.direction {
                PortDirection::Output => Some(ColumnEntry::Wire(wrapper.output_wire())),
                PortDirection::Input | PortDirection::Inout => {
                    wrapper.input_wire(col_def.name).map(ColumnEntry::Wire)
                }
                _ => None,
            };
            if let Some(e) = entry {
                entries[idx] = e;
            }
        }
        EntryArray::new(entries)
    }

    /// Rehydrate from row.
    fn primitive_rehydrate(
        _row: &Row<Self>,
        _store: &Store,
        _driver: &Driver,
        _key: &DriverKey,
        _config: &svql_common::Config,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Primitive> + 'static;

    /// Create a hierarchical report node from a match row
    ///
    /// Default implementation is identical to Netlist (displays ports).
    fn primitive_row_to_report_node(
        row: &Row<Self>,
        _store: &Store,
        driver: &Driver,
        key: &DriverKey,
        config: &svql_common::Config,
    ) -> crate::traits::display::ReportNode {
        use crate::traits::display::*;

        let type_name = std::any::type_name::<Self>();
        let short_name = type_name.rsplit("::").next().unwrap_or(type_name);

        let mut children = Vec::new();

        for port in Self::PORTS {
            if let Some(wire) = row.wire(port.name) {
                children.push(wire_to_report_node(
                    port.name,
                    wire,
                    port.direction,
                    driver,
                    key,
                    config,
                ));
            }
        }

        ReportNode {
            name: short_name.to_string(),
            type_name: type_name.to_string(),
            details: Some(format!("{:?}", Self::CELL_KIND)),
            source_loc: None,
            children,
        }
    }
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
        tracing::info!(
            "[PRIMITIVE] Starting primitive search for: {}",
            std::any::type_name::<T>()
        );

        let haystack_key = ctx.design_key();
        tracing::debug!("[PRIMITIVE] Haystack: {:?}", haystack_key);
        tracing::debug!("[PRIMITIVE] Target cell kind: {:?}", T::CELL_KIND);

        // Use the cached haystack design from context instead of calling get_design
        let haystack_container = ctx.haystack_design();
        tracing::trace!("[PRIMITIVE] Using cached haystack design");

        let index = haystack_container.index();
        tracing::debug!(
            "[PRIMITIVE] Searching design index for cells of type {:?}...",
            T::CELL_KIND
        );

        let cell_indices = index.cells_of_type_indices(T::CELL_KIND);
        let row_matches = if cell_indices.is_empty() {
            tracing::debug!(
                "[PRIMITIVE] No cells of type {:?} found in design",
                T::CELL_KIND
            );
            Vec::new()
        } else {
            tracing::debug!(
                "[PRIMITIVE] Found {} cells of type {:?}",
                cell_indices.len(),
                T::CELL_KIND
            );

            let filtered: Vec<_> = cell_indices
                .iter()
                .filter_map(|&cell_idx| {
                    let cell_wrapper = index.get_cell_by_index(cell_idx);
                    let cell = cell_wrapper.get();
                    let passes = T::cell_filter(cell.as_ref());
                    if !passes {
                        tracing::trace!(
                            "[PRIMITIVE] Cell filtered out: {:?}",
                            cell_wrapper.debug_index()
                        );
                        return None;
                    }
                    Some(T::resolve(cell_wrapper))
                })
                .collect();

            tracing::debug!("[PRIMITIVE] {} cells passed filter", filtered.len());
            filtered
        };

        tracing::info!(
            "[PRIMITIVE] Primitive search complete: {} total matches",
            row_matches.len()
        );
        Table::<Self>::new(row_matches)
    }

    fn internal_rehydrate<'a>(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
        config: &svql_common::Config,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Primitive> + Send + Sync + 'static,
    {
        Self::primitive_rehydrate(row, store, driver, key, config)
    }

    fn internal_row_to_report_node(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
        config: &svql_common::Config,
    ) -> crate::traits::display::ReportNode {
        Self::primitive_row_to_report_node(row, store, driver, key, config)
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

            const PORTS: &'static [svql_common::PortDecl] = &[
                $(svql_common::PortDecl::$direction(stringify!($port))),*
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
                _config: &svql_common::Config,
            ) -> Option<Self>
            where
                Self: $crate::traits::Component +
                      $crate::traits::PatternInternal<$crate::traits::kind::Primitive> +
                      Send + Sync + 'static,
            {
                Some($name {
                    $($port: row.wire(stringify!($port))?.clone()),*
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

            const PORTS: &'static [svql_common::PortDecl] = &[
                $(svql_common::PortDecl::$direction(stringify!($port))),*
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
                _config: &svql_common::Config,
            ) -> Option<Self>
            where
                Self: $crate::traits::Component +
                      $crate::traits::PatternInternal<$crate::traits::kind::Primitive> +
                      Send + Sync + 'static,
            {
                Some($name {
                    $(
                        $port: row.wire(stringify!($port))?.clone(),
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
    use svql_query::query_test;

    // Example primitive gate
    define_primitive!(TestAndGate, And, [(a, input), (b, input), (y, output)]);

    // small_and_tree has 3 AND gates total
    query_test!(
        name: test_and_gate_small_tree,
        query: TestAndGate,
        haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
        expect: 3  // Automatically deduplicated
    );

    #[test]
    fn test_and_gate_rehydration() -> Result<(), Box<dyn std::error::Error>> {
        use crate::test_harness::setup_test_logging;

        setup_test_logging();

        let driver = Driver::new_workspace()?;
        let config = Config::builder().build();
        let key = DriverKey::new(
            "examples/fixtures/basic/and/verilog/small_and_tree.v",
            "small_and_tree",
        );

        let store = svql_query::run_query::<TestAndGate>(&driver, &key, &config)?;
        let table = store.get::<TestAndGate>().expect("Table should exist");

        // Verify we can rehydrate and access fields
        let (_ref, first_row) = table.rows().next().expect("Should have at least one match");
        let gate = <TestAndGate as svql_query::traits::Pattern>::rehydrate(
            &first_row, &store, &driver, &key, &config,
        )
        .expect("Should rehydrate");

        // Access fields to prove they work
        assert!(
            gate.a.cell_id().expect("Wire must be a cell").storage_key() > 0,
            "Input A should have valid ID"
        );
        assert!(
            gate.b.cell_id().expect("Wire must be a cell").storage_key() > 0,
            "Input B should have valid ID"
        );
        assert!(
            gate.y.cell_id().expect("Wire must be a cell").storage_key() > 0,
            "Output Y should have valid ID"
        );

        Ok(())
    }
}
