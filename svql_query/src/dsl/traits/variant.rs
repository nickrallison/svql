//! Core logic for polymorphic pattern selection.
//!
//! Implements the concatenation and dispatch logic for patterns
//! that offer multiple alternative implementations.

use crate::prelude::*;

/// Describes one variant arm for lookup during rehydration.
///
/// Contains metadata about each variant arm to enable runtime dispatch
/// during rehydration.
#[derive(Debug, Clone, Copy)]
pub struct VariantArm {
    /// The `TypeId` of the inner pattern type.
    pub type_id: std::any::TypeId,
    /// Human-readable type name for debugging.
    pub type_name: &'static str,
}

/// Defines an interface for patterns that support multiple concrete implementations.
pub trait Variant: Sized + Component<Kind = kind::Variant> + Send + Sync + 'static {
    /// Number of variant arms
    const NUM_VARIANTS: usize;

    /// Common interface ports (macro-generated)
    const COMMON_PORTS: &'static [PortDecl];

    /// Port mappings for each variant arm (macro-generated)
    const PORT_MAPPINGS: &'static [&'static [PortMap]];

    /// Variant arm metadata (macro-generated)
    const VARIANT_ARMS: &'static [VariantArm];

    /// Dependencies (macro-generated)
    const DEPENDANCIES: &'static [&'static ExecInfo];

    /// Schema accessor (macro generates this with `OnceLock` pattern)
    fn variant_schema() -> &'static crate::session::PatternSchema {
        static SCHEMA: std::sync::OnceLock<crate::session::PatternSchema> =
            std::sync::OnceLock::new();
        SCHEMA.get_or_init(|| {
            let defs = Self::variant_to_defs();
            let defs_static: &'static [ColumnDef] = Box::leak(defs.into_boxed_slice());
            crate::session::PatternSchema::new(defs_static)
        })
    }

    /// Convert declarations to column definitions
    #[must_use]
    fn variant_to_defs() -> Vec<ColumnDef> {
        let mut defs = vec![
            ColumnDef::metadata("discriminant"),
            ColumnDef::sub::<()>("inner_ref"),
        ];

        defs.extend(
            Self::COMMON_PORTS.iter().map(|p| {
                ColumnDef::new(p.name, ColumnKind::Wire, false).with_direction(p.direction)
            }),
        );

        defs
    }

    /// Concatenate results from all variant arms into a unified table.
    ///
    /// This is the core operation for variants - it unions the results from
    /// each inner type, mapping their ports to the common interface and
    /// tracking which arm each row came from via the discriminant.
    ///
    /// # Errors
    ///
    /// Returns a `QueryError` if:
    /// * The variant schema is missing critical column definitions (e.g., "discriminant").
    /// * A variant arm's port mapping fails to resolve.
    fn concatenate(
        ctx: &ExecutionContext,
        dep_tables: &[&(dyn AnyTable + Send + Sync)],
    ) -> Result<Table<Self>, QueryError> {
        tracing::info!(
            "[VARIANT] Starting variant concatenation for: {}",
            std::any::type_name::<Self>()
        );

        let schema = Self::variant_schema();
        let mut all_entries = Vec::new();

        tracing::debug!("[VARIANT] Number of variant arms: {}", Self::NUM_VARIANTS);
        for (i, table) in dep_tables.iter().enumerate() {
            tracing::debug!(
                "  [{}] {}: {} rows",
                i,
                Self::VARIANT_ARMS[i].type_name,
                table.len()
            );
        }

        // Column indices (cached once)
        let discrim_idx = schema
            .index_of("discriminant")
            .ok_or_else(|| QueryError::SchemaLut("discriminant".to_string()))?;
        let inner_ref_idx = schema
            .index_of("inner_ref")
            .ok_or_else(|| QueryError::SchemaLut("inner_ref".to_string()))?;

        for (variant_idx, table) in dep_tables.iter().enumerate() {
            let variant_name = Self::VARIANT_ARMS[variant_idx].type_name;
            tracing::debug!(
                "[VARIANT] Processing variant arm {}/{}: {} ({} rows)",
                variant_idx + 1,
                Self::NUM_VARIANTS,
                variant_name,
                table.len()
            );

            let port_maps = Self::PORT_MAPPINGS[variant_idx];
            tracing::trace!(
                "[VARIANT] Port mappings for {}: {} mappings",
                variant_name,
                port_maps.len()
            );

            for row_idx in 0..table.len() as u32 {
                let mut entry = EntryArray::with_capacity(2 + Self::COMMON_PORTS.len());

                // Initialize all wire entries to None
                for i in 0..(2 + Self::COMMON_PORTS.len()) {
                    entry.entries[i] = ColumnEntry::Null;
                }

                // 1. Set discriminant
                entry.entries[discrim_idx] =
                    ColumnEntry::Metadata(PhysicalCellId::new(variant_idx as u32));

                // 2. Set inner_ref (row index in the variant arm's table)
                entry.entries[inner_ref_idx] = ColumnEntry::Sub(row_idx);

                // 3. Map common ports from inner table using path resolution
                for mapping in port_maps {
                    if let Some(col_idx) = schema.index_of(mapping.common_port) {
                        let wire = table.resolve_path(row_idx as usize, mapping.inner, ctx);
                        entry.entries[col_idx] =
                            wire.map(ColumnEntry::Wire).unwrap_or(ColumnEntry::Null);
                    }
                }

                all_entries.push(entry);
            }

            tracing::debug!(
                "[VARIANT] Processed {} rows from variant arm: {}",
                table.len(),
                variant_name
            );
        }

        // Apply automatic deduplication
        let before_dedup = all_entries.len();
        Self::apply_deduplication(&mut all_entries);
        if before_dedup != all_entries.len() {
            tracing::debug!(
                "[VARIANT] Deduplication: {} -> {} entries ({} removed)",
                before_dedup,
                all_entries.len(),
                before_dedup - all_entries.len()
            );
        }

        tracing::info!(
            "[VARIANT] Variant concatenation complete: {} total matches",
            all_entries.len()
        );
        Table::new(all_entries)
    }

    /// Apply automatic deduplication.
    fn apply_deduplication(entries: &mut Vec<EntryArray>) {
        crate::traits::apply_deduplication(entries);
    }

    /// Pre-loads all possible variant designs into the driver.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the underlying variants fail to preload.
    fn preload_driver(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Dispatches rehydration to the specific implementation matched in the row.
    fn variant_rehydrate(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Variant> + 'static;

    /// Create a hierarchical report node from a match row
    ///
    /// Recursive implementation dispatches to the active variant's display
    /// logic using macro-generated metadata.
    fn variant_row_to_report_node(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> crate::traits::display::ReportNode {
        use crate::traits::display::*;

        let schema = Self::variant_schema();
        let type_name = std::any::type_name::<Self>();
        let short_name = type_name.rsplit("::").next().unwrap_or(type_name);

        // Get discriminant and inner_ref from schema
        let discrim = schema
            .index_of("discriminant")
            .and_then(|idx| row.entry_array.entries.get(idx))
            .and_then(|e| e.as_u32())
            .unwrap_or(0);

        let inner_ref = schema
            .index_of("inner_ref")
            .and_then(|idx| row.entry_array.entries.get(idx))
            .and_then(|e| e.as_u32())
            .unwrap_or(0);

        // Dispatch to active variant using metadata
        if (discrim as usize) < Self::NUM_VARIANTS {
            let arm = &Self::VARIANT_ARMS[discrim as usize];

            if let Some(mut node) = store
                .get_from_tid(arm.type_id)
                .and_then(|table| table.row_to_report_node(inner_ref as usize, store, driver, key))
            {
                // Add variant type to details
                node.details = Some(format!("{}::{}", short_name, arm.type_name));
                return node;
            }
        }

        // Fallback for invalid discriminant
        ReportNode {
            name: short_name.to_string(),
            type_name: type_name.to_string(),
            details: Some("Unknown variant".to_string()),
            source_loc: None,
            children: vec![],
        }
    }
}

impl<T> PatternInternal<kind::Variant> for T
where
    T: Variant + Component<Kind = kind::Variant> + Send + Sync + 'static,
{
    const DEFS: &'static [ColumnDef] = &[]; // Placeholder

    const SCHEMA_SIZE: usize = 2 + T::COMMON_PORTS.len(); // discriminant + inner_ref + common ports

    const EXEC_INFO: &'static crate::session::ExecInfo = &crate::session::ExecInfo {
        type_id: std::any::TypeId::of::<T>(),
        type_name: std::any::type_name::<T>(),
        search_function: |ctx| {
            search_table_any::<T>(ctx, <T as PatternInternal<kind::Variant>>::search_table)
        },
        nested_dependancies: T::DEPENDANCIES,
    };

    fn internal_schema() -> &'static crate::session::PatternSchema {
        T::variant_schema()
    }

    fn preload_driver(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        <T as Variant>::preload_driver(driver, design_key, config)
    }

    fn search_table(ctx: &ExecutionContext) -> Result<Table<Self>, QueryError>
    where
        Self: Send + Sync + 'static,
    {
        // 1. Gather dependency tables (one per variant arm)
        let mut dep_tables: Vec<&(dyn AnyTable + Send + Sync)> =
            Vec::with_capacity(T::NUM_VARIANTS);

        for (i, dep_info) in T::DEPENDANCIES.iter().enumerate() {
            let table = ctx.get_any_table(dep_info.type_id).ok_or_else(|| {
                QueryError::MissingDependency(format!(
                    "Variant arm {}: {} (TypeId {:?})",
                    i, dep_info.type_name, dep_info.type_id
                ))
            })?;
            dep_tables.push(table);
        }

        // 2. Concatenate results from all arms
        T::concatenate(ctx, &dep_tables)
    }

    fn internal_rehydrate<'a>(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Variant> + Send + Sync + 'static,
    {
        Self::variant_rehydrate(row, store, driver, key)
    }

    fn internal_row_to_report_node(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> crate::traits::display::ReportNode {
        Self::variant_row_to_report_node(row, store, driver, key)
    }
}

/// Test utilities and examples for variants.
#[allow(unused)]
mod test {

    use crate::{
        Wire,
        prelude::PortDecl,
        session::ExecInfo,
        traits::{Netlist, Pattern, composite::Composite},
    };

    use super::{
        Component, Driver, DriverKey, Port, PortMap, Row, Store, Variant, VariantArm, kind,
    };

    use crate::traits::composite::test::And2Gates;
    use crate::traits::netlist::test::AndGate;

    use svql_common::Selector;
    use svql_query::query_test;

    /// A polymorphic pattern representing either an `AndGate` or `And2Gates`.
    #[derive(Debug, Clone, Variant)]
    #[variant_ports(input(a), input(b), output(y))]
    pub enum AndOrAnd2 {
        /// Direct match on a single AND gate.
        #[map(a = ["a"], b = ["b"], y = ["y"])]
        AndGate(AndGate),
        /// Match on a composite structure of two AND gates.
        #[map(a = ["a"], b = ["b"], y = ["y"])]
        And2Gates(And2Gates),
    }

    query_test!(
        name: test_and_mixed_and_tree,
        query: AndOrAnd2,
        haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
        expect: 5  // 3 AndGate + 2 And2Gates
    );

    /// Manually implemented variant for testing.
    #[derive(Debug, Clone)]
    pub enum ManualAndOrAnd2 {
        /// Direct match on a single AND gate.
        AndGate(AndGate),
        /// Match on a composite structure of two AND gates.
        And2Gates(And2Gates),
    }

    impl Component for ManualAndOrAnd2 {
        type Kind = kind::Variant;
    }

    impl Variant for ManualAndOrAnd2 {
        const NUM_VARIANTS: usize = 2;

        const COMMON_PORTS: &'static [PortDecl] = &[
            PortDecl::input("a"),
            PortDecl::input("b"),
            PortDecl::output("y"),
        ];

        const PORT_MAPPINGS: &'static [&'static [PortMap]] = &[
            &[
                PortMap::new("a", Selector::static_path(&["a"])),
                PortMap::new("b", Selector::static_path(&["b"])),
                PortMap::new("y", Selector::static_path(&["y"])),
            ],
            &[
                PortMap::new("a", Selector::static_path(&["a"])),
                PortMap::new("b", Selector::static_path(&["b"])),
                PortMap::new("y", Selector::static_path(&["y"])),
            ],
        ];

        const VARIANT_ARMS: &'static [VariantArm] = &[
            VariantArm {
                type_id: std::any::TypeId::of::<AndGate>(),
                type_name: "AndGate",
            },
            VariantArm {
                type_id: std::any::TypeId::of::<And2Gates>(),
                type_name: "And2Gates",
            },
        ];

        const DEPENDANCIES: &'static [&'static ExecInfo] = &[
            <AndGate as Pattern>::EXEC_INFO,
            <And2Gates as Pattern>::EXEC_INFO,
        ];

        fn variant_rehydrate(
            row: &Row<Self>,
            store: &Store,
            driver: &Driver,
            key: &DriverKey,
        ) -> Option<Self> {
            let schema = Self::schema();
            let discrim = row
                .entry_array
                .entries
                .get(schema.index_of("discriminant")?)?
                .as_u32()?;
            let inner_row_idx = row
                .entry_array
                .entries
                .get(schema.index_of("inner_ref")?)?
                .as_u32()?;

            match discrim {
                0 => {
                    let table = store.get::<AndGate>()?;
                    let inner_row = table.row(inner_row_idx)?;
                    let inner = <AndGate as Pattern>::rehydrate(&inner_row, store, driver, key)?;
                    Some(Self::AndGate(inner))
                }
                1 => {
                    let table = store.get::<And2Gates>()?;
                    let inner_row = table.row(inner_row_idx)?;
                    let inner = <And2Gates as Pattern>::rehydrate(&inner_row, store, driver, key)?;
                    Some(Self::And2Gates(inner))
                }
                _ => None,
            }
        }

        fn preload_driver(
            driver: &Driver,
            design_key: &DriverKey,
            config: &svql_common::Config,
        ) -> Result<(), Box<dyn std::error::Error>> {
            <AndGate as Pattern>::preload_driver(driver, design_key, config)?;
            <And2Gates as Pattern>::preload_driver(driver, design_key, config)?;
            Ok(())
        }
    }

    query_test!(
        name: test_manual_variant_small_tree,
        query: ManualAndOrAnd2,
        haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
        expect: 5
    );
}
