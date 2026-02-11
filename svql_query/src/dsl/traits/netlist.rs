//! Netlist component traits and utilities.
//!
//! Provides traits for components defined via external HDL files.

use crate::prelude::*;
use prjunnamed_netlist::Value;
use tracing::debug;

fn value_to_cell_id(value: &Value) -> Option<u32> {
    match value.as_net() {
        Some(net) => net.as_cell_index().map(|idx| idx as u32).ok(),
        None => None,
    }
}

/// Trait for netlist-based pattern components.
///
/// Implemented by types generated with `#[netlist]`. Provides access to
/// the source file path and module name.
pub trait Netlist: Sized + Component<Kind = kind::Netlist> + Send + Sync + 'static {
    /// The module name within the source file.
    const MODULE_NAME: &'static str;

    /// Path to the netlist source file (.v, .il, or .json).
    const FILE_PATH: &'static str;

    /// Port declarations (macro-generated)
    const PORTS: &'static [Port];

    /// Schema accessor (macro generates this with `OnceLock` pattern)
    fn netlist_schema() -> &'static crate::session::PatternSchema {
        static SCHEMA: std::sync::OnceLock<crate::session::PatternSchema> =
            std::sync::OnceLock::new();
        SCHEMA.get_or_init(|| {
            let mut defs = Self::ports_to_defs();

            // Load needle design to discover internal cells
            let result = std::panic::catch_unwind(|| Self::discover_internal_cells());

            match result {
                Ok(Ok(internal_defs)) => {
                    tracing::debug!(
                        "[NETLIST] {} discovered {} internal cells",
                        std::any::type_name::<Self>(),
                        internal_defs.len()
                    );
                    defs.extend(internal_defs);
                }
                Ok(Err(e)) => {
                    tracing::warn!(
                        "[NETLIST] {} failed to load needle during schema init: {}",
                        std::any::type_name::<Self>(),
                        e
                    );
                }
                Err(panic_val) => {
                    tracing::error!(
                        "[NETLIST] {} panicked during needle loading: {:?}",
                        std::any::type_name::<Self>(),
                        panic_val
                    );
                }
            }

            let defs_static: &'static [ColumnDef] = Box::leak(defs.into_boxed_slice());
            crate::session::PatternSchema::new(defs_static)
        })
    }

    /// Convert port declarations to column definitions
    #[must_use]
    fn ports_to_defs() -> Vec<ColumnDef> {
        Self::PORTS
            .iter()
            .map(|p| ColumnDef::new(p.name, ColumnKind::Cell, false).with_direction(p.direction))
            .collect()
    }

    /// Load the needle and extract metadata columns for internal cells.
    ///
    /// Discovers internal logic gates in the needle design and creates
    /// metadata columns for them. This allows storing the haystack cell IDs
    /// for internal cells in the result table.
    fn discover_internal_cells() -> Result<Vec<ColumnDef>, Box<dyn std::error::Error>> {
        let ym = YosysModule::new(Self::FILE_PATH, Self::MODULE_NAME)?;

        let yosys = match which::which("yosys") {
            Ok(path) => path,
            Err(_) => {
                tracing::debug!(
                    "[NETLIST] Yosys not found, skipping internal cell discovery for {}",
                    std::any::type_name::<Self>()
                );
                return Ok(Vec::new());
            }
        };

        let design = ym.import_design_yosys(&ModuleConfig::default(), &yosys)?;
        let index = GraphIndex::build(&design);

        let mut internal_defs = Vec::new();

        for i in 0..index.num_cells() {
            let cell_idx = GraphNodeIdx::new(i as u32);
            let cell_wrapper = index.get_cell_by_index(cell_idx);
            let kind = cell_wrapper.cell_type();

            // Only store internal logic gates, not I/O ports
            if kind.is_logic_gate() {
                let debug_id = cell_wrapper.debug_index();
                let col_name: &'static str =
                    Box::leak(format!("__internal_cell_{}", debug_id).into_boxed_str());
                internal_defs.push(ColumnDef::metadata(col_name));
            }
        }

        Ok(internal_defs)
    }

    // /// Returns a reference to the lazily-initialized driver key for this netlist.
    // ///
    // /// The driver key is constructed once on the first call and cached in a static OnceLock.
    // fn driver_key() -> &'static DriverKey {
    //     static DRIVER_KEY: std::sync::OnceLock<DriverKey> = std::sync::OnceLock::new();
    //     DRIVER_KEY.get_or_init(|| {
    //         debug!(
    //             "Creating driver key for netlist: {}, file: {}",
    //             Self::MODULE_NAME,
    //             Self::FILE_PATH
    //         );
    //         DriverKey::new(Self::FILE_PATH, Self::MODULE_NAME.to_string())
    //     })
    // }

    /// Returns the driver key for this netlist.
    fn driver_key() -> DriverKey {
        debug!(
            "Creating driver key for netlist: {}, file: {}",
            Self::MODULE_NAME,
            Self::FILE_PATH
        );
        DriverKey::new(Self::FILE_PATH, Self::MODULE_NAME.to_string())
    }

    #[must_use]
    fn resolve(
        assignment: &SingleAssignment,
        needle_index: &GraphIndex<'_>,
        haystack_index: &GraphIndex<'_>,
    ) -> EntryArray {
        let schema = Self::netlist_schema();
        let mut entries = vec![ColumnEntry::Null; schema.defs.len()];

        for (n_node, h_node) in assignment.needle_mapping() {
            let needle_wrapper = needle_index.get_cell_by_index(*n_node);

            // TRANSLATION: Map local search node to stable physical ID
            let haystack_physical = haystack_index.resolve_physical(*h_node);

            match needle_wrapper.get() {
                prjunnamed_netlist::Cell::Input(name, _) => {
                    let err_msg = format!(
                        "Needle Cell name: {name} should exist in schema for: {} at {}",
                        Self::MODULE_NAME,
                        Self::FILE_PATH
                    );
                    let col_idx = schema.index_of(&name).expect(&err_msg);
                    entries[col_idx] = ColumnEntry::cell(haystack_physical);
                }
                prjunnamed_netlist::Cell::Output(name, output_value) => {
                    let err_msg = format!(
                        "Needle Cell name: {name} should exist in schema for: {} at {}",
                        Self::MODULE_NAME,
                        Self::FILE_PATH
                    );
                    let col_idx = schema.index_of(&name).expect(&err_msg);
                    let needle_output_driver_id: u32 =
                        value_to_cell_id(&output_value).expect("Output should have driver");
                    let haystack_output_driver = assignment
                        .needle_mapping()
                        .iter()
                        .find(|(n_idx, _h_idx)| {
                            needle_index
                                .get_cell_by_index(**n_idx)
                                .debug_index()
                                .storage_key()
                                == needle_output_driver_id
                        })
                        .map(|(_n_idx, h_idx)| h_idx)
                        .expect("Should find haystack driver for output");

                    entries[col_idx] = ColumnEntry::cell(
                        haystack_index.resolve_physical(*haystack_output_driver),
                    );
                }
                _ => {
                    // Internal cell — store in metadata column if we have one
                    let needle_debug_id = needle_wrapper.debug_index();
                    let col_name = format!("__internal_{}", needle_debug_id);

                    if let Some(col_idx) = schema.index_of(&col_name) {
                        entries[col_idx] = ColumnEntry::Metadata(haystack_physical);

                        tracing::trace!(
                            "[NETLIST] Stored internal cell mapping: needle[{}] -> haystack[{}]",
                            needle_debug_id,
                            haystack_physical.storage_key()
                        );
                    }
                }
            }
        }
        EntryArray::new(entries)
    }

    fn netlist_rehydrate(
        _row: &Row<Self>,
        _store: &Store,
        _driver: &Driver,
        _key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Netlist> + Send + Sync + 'static;

    /// Create a hierarchical report node from a match row
    ///
    /// Default implementation uses the PORTS schema to display all ports
    /// with their source locations, plus any discovered internal cells.
    fn netlist_row_to_report_node(
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

        // 1. Show all ports from PORTS schema
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

        // 2. Show internal cells from metadata columns
        let schema = Self::netlist_schema();
        let haystack_container = driver.get_design(key, &config.haystack_options).ok();

        for (idx, col_def) in schema.columns().iter().enumerate() {
            // Skip non-metadata columns (ports already shown above)
            if !col_def.kind.is_metadata() {
                continue;
            }

            // Only show columns that are internal cells
            if !col_def.name.starts_with("__internal_cell_") {
                continue;
            }

            // Get the haystack cell ID from the row
            let haystack_cell_id = match row.entry_array().entries.get(idx) {
                Some(ColumnEntry::Metadata(id)) => *id,
                _ => continue,
            };

            // Try to get source location from the haystack
            let source_loc = haystack_container.as_ref().and_then(|container| {
                container
                    .index()
                    .get_cell_by_id(haystack_cell_id.storage_key() as usize)
                    .and_then(|cell_wrapper| cell_wrapper.get_source())
            });

            // Try to get the needle cell kind from the debug_id
            let needle_debug_id: usize = col_def
                .name
                .strip_prefix("__internal_cell_")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            // Load needle to get cell type info
            let needle_key = Self::driver_key();
            let cell_kind_str = driver
                .get_design(&needle_key, &config.needle_options)
                .ok()
                .and_then(|container| {
                    container
                        .index()
                        .get_cell_by_id(needle_debug_id)
                        .map(|cell_wrapper| format!("{:?}", cell_wrapper.cell_type()))
                })
                .unwrap_or_else(|| "Unknown".to_string());

            children.push(ReportNode {
                name: format!("internal_{}", needle_debug_id),
                type_name: cell_kind_str,
                details: Some(format!("cell_{}", haystack_cell_id)),
                source_loc,
                children: vec![],
            });
        }

        ReportNode {
            name: short_name.to_string(),
            type_name: type_name.to_string(),
            details: None,
            source_loc: None,
            children,
        }
    }
}

impl<T> PatternInternal<kind::Netlist> for T
where
    T: Netlist + Component<Kind = kind::Netlist> + Send + Sync + 'static,
{
    const DEFS: &'static [ColumnDef] = &[]; // Placeholder, not used anymore

    const SCHEMA_SIZE: usize = T::PORTS.len();

    const EXEC_INFO: &'static crate::session::ExecInfo = &crate::session::ExecInfo {
        type_id: std::any::TypeId::of::<T>(),
        type_name: std::any::type_name::<T>(),
        search_function: |ctx| {
            search_table_any::<T>(ctx, <T as PatternInternal<kind::Netlist>>::search_table)
        },
        nested_dependancies: &[],
    };

    fn internal_schema() -> &'static crate::session::PatternSchema {
        T::netlist_schema()
    }

    fn preload_driver(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        driver.preload_design(&Self::driver_key(), &config.needle_options)?;
        driver.preload_design(design_key, &config.haystack_options)?;
        Ok(())
    }

    fn search_table(ctx: &ExecutionContext) -> Result<Table<Self>, QueryError>
    where
        Self: Send + Sync + 'static,
    {
        tracing::info!(
            "[NETLIST] Starting netlist search for: {}",
            std::any::type_name::<Self>()
        );

        let needle_key = Self::driver_key();
        let haystack_key = ctx.design_key();

        tracing::debug!("[NETLIST] Needle: {:?}", needle_key);
        tracing::debug!("[NETLIST] Haystack: {:?}", haystack_key);

        // Load needle design once, use cached haystack design from context
        tracing::debug!("[NETLIST] Loading needle design...");
        let needle_container = ctx
            .driver()
            .get_design(&needle_key, &ctx.config().needle_options)
            .map_err(|e| QueryError::needle_load(e.to_string()))?;
        tracing::debug!("[NETLIST] Needle design loaded");

        let haystack_container = ctx.haystack_design();
        tracing::debug!("[NETLIST] Using cached haystack design");

        tracing::info!("[NETLIST] Starting subgraph matching...");
        let assignments = subgraph::SubgraphMatcher::enumerate_with_indices(
            needle_container.design(),
            haystack_container.design(),
            needle_container.index(),
            haystack_container.index(),
            needle_key.module_name().to_string(),
            haystack_key.module_name().to_string(),
            ctx.config(),
        );
        tracing::info!(
            "[NETLIST] Subgraph matching complete: {} assignments found",
            assignments.items.len()
        );

        tracing::debug!("[NETLIST] Resolving assignments to table rows...");
        let mut row_matches: Vec<EntryArray> = assignments
            .items
            .iter()
            .map(|assignment| {
                Self::resolve(
                    assignment,
                    needle_container.index(),
                    haystack_container.index(),
                )
            })
            .collect();
        tracing::debug!(
            "[NETLIST] {} rows created from assignments",
            row_matches.len()
        );

        // Apply automatic row-level deduplication
        let before_dedup = row_matches.len();
        crate::traits::apply_deduplication(&mut row_matches);
        if before_dedup != row_matches.len() {
            tracing::debug!(
                "[NETLIST] Deduplication: {} -> {} rows ({} removed)",
                before_dedup,
                row_matches.len(),
                before_dedup - row_matches.len()
            );
        }

        let table = Table::<Self>::new(row_matches)?;
        tracing::info!(
            "[NETLIST] Netlist search complete: {} total matches",
            table.len()
        );
        Ok(table)
    }

    fn internal_rehydrate<'a>(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Netlist> + Send + Sync + 'static,
    {
        Self::netlist_rehydrate(row, store, driver, key)
    }

    fn internal_row_to_report_node(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> crate::traits::display::ReportNode {
        Self::netlist_row_to_report_node(row, store, driver, key)
    }
}

#[allow(unused)]
pub(crate) mod test {

    use crate::Wire;

    use super::{Component, Driver, DriverKey, Netlist, Port, Row, Store, kind};

    use svql_query::query_test;

    #[derive(Debug, Clone, Netlist)]
    #[netlist(
        file = "examples/fixtures/basic/and/verilog/and_gate.v",
        module = "and_gate"
    )]
    pub struct AndGate {
        #[port(input)]
        pub a: Wire,
        #[port(input)]
        pub b: Wire,
        #[port(output)]
        pub y: Wire,
    }

    query_test!(
        name: test_and_mixed_and_tree,
        query: AndGate,
        haystack: ("examples/fixtures/basic/and/json/mixed_and_tree.json", "mixed_and_tree"),
        expect: 3  // Automatically deduplicated
    );

    // --- Reference Implementation (Manual) ---

    #[derive(Debug, Clone)]
    pub struct ManualAndGate {
        pub a: Wire,
        pub b: Wire,
        pub y: Wire,
    }

    impl Component for ManualAndGate {
        type Kind = kind::Netlist;
    }

    impl Netlist for ManualAndGate {
        const MODULE_NAME: &'static str = "and_gate";
        const FILE_PATH: &'static str = "examples/fixtures/basic/and/verilog/and_gate.v";

        const PORTS: &'static [Port] = &[Port::input("a"), Port::input("b"), Port::output("y")];

        fn netlist_rehydrate(
            row: &Row<Self>,
            _store: &Store,
            _driver: &Driver,
            _key: &DriverKey,
        ) -> Option<Self> {
            Some(Self {
                a: row.wire("a")?,
                b: row.wire("b")?,
                y: row.wire("y")?,
            })
        }
    }

    query_test!(
        name: test_manual_and_gate_small_tree,
        query: ManualAndGate,
        haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
        expect: 3  // With default Dedupe::Inner, identical rows are deduplicated
    );
}
