//! Primitive component traits for direct cell type matching.
//!
//! Unlike netlists which use subgraph matching against external files,
//! primitives match directly against cell types in the design index.

use crate::{
    prelude::*,
    session::{ColumnEntry, EntryArray, ExecutionContext, Port, QueryError, Row, Store, Table},
    traits::{Component, PatternInternal, kind, search_table_any},
    wire::WireRef,
};
use std::sync::Arc;
use svql_driver::{Driver, DriverKey};

// svql_query/src/traits/primitive.rs

/// Helper to extract wire reference from a Value
fn value_to_wire_ref(value: &prjunnamed_netlist::Value) -> Option<WireRef> {
    // Try to get the first net from the value
    match value.iter().take(1).next() {
        Some(net) => net_to_wire_ref(&net),
        None => {
            // No nets means it's likely a constant value
            // For now, we'll treat empty values as constant false
            // TODO: Better constant detection once prjunnamed_netlist API is clearer
            Some(WireRef::Constant(false))
        }
    }
}

/// Helper to extract wire reference from a Net
fn net_to_wire_ref(net: &prjunnamed_netlist::Net) -> Option<WireRef> {
    // Try as cell first
    if let Ok(cell_idx) = net.as_cell_index() {
        return Some(WireRef::Cell(PhysicalCellId::new(cell_idx as u32)));
    }

    // If not a cell, assume it's an input port or constant
    // For now, we'll create a synthetic name based on the net
    // TODO: Investigate if there's a way to get the actual input name from the net
    Some(WireRef::PrimaryPort(Arc::from(format!("net_{:?}", net))))
}

/// Helper to extract wire reference from a ControlNet
fn control_net_to_wire_ref(cnet: &prjunnamed_netlist::ControlNet) -> Option<WireRef> {
    net_to_wire_ref(&cnet.net())
}

/// Extract the wire reference for a specific input port by name from a cell.
fn extract_input_port(cell: &prjunnamed_netlist::Cell, port_name: &str) -> Option<WireRef> {
    use prjunnamed_netlist::Cell;

    match cell {
        // 2-input gates
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
        | Cell::SModFloor(a, b) => match port_name {
            "a" => value_to_wire_ref(a),
            "b" => value_to_wire_ref(b),
            _ => None,
        },

        // 1-input gates
        Cell::Not(a) | Cell::Buf(a) => match port_name {
            "a" => value_to_wire_ref(a),
            _ => None,
        },

        // Mux: (sel, a, b)
        Cell::Mux(s, a, b) => match port_name {
            "sel" => net_to_wire_ref(s),
            "a" => value_to_wire_ref(a),
            "b" => value_to_wire_ref(b),
            _ => None,
        },

        // Adc: (a, b, carry_in)
        Cell::Adc(a, b, ci) => match port_name {
            "a" => value_to_wire_ref(a),
            "b" => value_to_wire_ref(b),
            "carry_in" | "cin" | "ci" => net_to_wire_ref(ci),
            _ => None,
        },

        // AIG: (a, b) - both are ControlNets
        Cell::Aig(a, b) => match port_name {
            "a" => control_net_to_wire_ref(a),
            "b" => control_net_to_wire_ref(b),
            _ => None,
        },

        // Shift operations
        Cell::Shl(a, b, _) | Cell::UShr(a, b, _) | Cell::SShr(a, b, _) | Cell::XShr(a, b, _) => {
            match port_name {
                "a" => value_to_wire_ref(a),
                "b" => value_to_wire_ref(b),
                _ => None,
            }
        }

        Cell::Dff(ff) => {
            match port_name {
                "clk" => net_to_wire_ref(&ff.clock.net()),
                "d" | "data_in" => value_to_wire_ref(&ff.data),
                // Handle various names for enable
                "en" | "enable" | "write_en" => {
                    if ff.enable.net().is_cell() {
                        net_to_wire_ref(&ff.enable.net())
                    } else {
                        None
                    }
                }
                "reset" | "rst" | "srst" => {
                    if ff.reset.net().is_cell() {
                        net_to_wire_ref(&ff.reset.net())
                    } else {
                        None
                    }
                }
                "reset_n" | "rst_n" | "clear" | "arst" | "arst_n" => {
                    if let Some(net) = ff.clear.nets().first() {
                        if net.is_cell() {
                            net_to_wire_ref(net)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }

        // Add memory and other complex cells as needed
        _ => None,
    }
}

/// Resolve primitive ports by extracting inputs from the cell structure.
///
/// Creates an entry array aligned with the schema, where each port is populated
/// only if it exists in the cell and has a valid driving wire reference.
pub fn resolve_primitive_ports(
    cell: &prjunnamed_netlist::Cell,
    debug_index: usize,
    schema: &PatternSchema,
) -> EntryArray {
    let cell_id = PhysicalCellId::new(debug_index as u32);
    let schema_size = schema.defs.len();

    // Initialize all entries as None
    let mut entries = vec![ColumnEntry::Wire { value: None }; schema_size];

    // For each column in the schema, try to extract the corresponding port value
    for (idx, col_def) in schema.defs.iter().enumerate() {
        let port_name = col_def.name;

        let wire_ref_opt = match col_def.direction {
            PortDirection::Output => {
                // Output port typically maps to the cell itself
                Some(WireRef::Cell(cell_id))
            }
            PortDirection::Input | PortDirection::Inout => {
                // Extract input value (might be cell, port, or constant)
                extract_input_port(cell, port_name)
            }
            PortDirection::None => {
                // Non-port column (shouldn't happen in primitives), skip
                None
            }
        };

        entries[idx] = ColumnEntry::Wire {
            value: wire_ref_opt,
        };
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
    fn resolve(cell: &prjunnamed_netlist::Cell, debug_index: usize) -> EntryArray {
        resolve_primitive_ports(cell, debug_index, Self::primitive_schema())
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

    /// Create a hierarchical report node from a match row
    ///
    /// Default implementation is identical to Netlist (displays ports).
    fn primitive_row_to_report_node(
        row: &Row<Self>,
        _store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> crate::traits::display::ReportNode {
        use crate::traits::display::*;

        let config = Config::default();
        let type_name = std::any::type_name::<Self>();
        let short_name = type_name.rsplit("::").next().unwrap_or(type_name);

        let mut children = Vec::new();

        for port in Self::PORTS {
            if let Some(wire) = row.wire(port.name) {
                children.push(wire_to_report_node(
                    port.name,
                    &wire,
                    port.direction,
                    driver,
                    key,
                    &config,
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
                    let passes = T::cell_filter(cell);
                    if !passes {
                        tracing::trace!(
                            "[PRIMITIVE] Cell filtered out: {:?}",
                            cell_wrapper.debug_index()
                        );
                        return None;
                    }
                    Some(T::resolve(cell, cell_wrapper.debug_index().storage_key() as usize))
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
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Primitive> + Send + Sync + 'static,
    {
        Self::primitive_rehydrate(row, store, driver, key)
    }

    fn internal_row_to_report_node(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> crate::traits::display::ReportNode {
        Self::primitive_row_to_report_node(row, store, driver, key)
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
        let first_row = table.rows().next().expect("Should have at least one match");
        let gate = <TestAndGate as svql_query::traits::Pattern>::rehydrate(
            &first_row, &store, &driver, &key,
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
