use svql_driver::{Driver, DriverKey};

use crate::{
    prelude::{ColumnDef, ColumnKind, QueryError, Table},
    session::{AnyTable, ColumnEntry, EntryArray, ExecInfo, ExecutionContext, Port, PortMap, Row, Store},
    traits::{Component, PatternInternal, kind, search_table_any},
};

/// Describes one variant arm for lookup during rehydration.
///
/// Contains metadata about each variant arm to enable runtime dispatch
/// during rehydration.
#[derive(Debug, Clone, Copy)]
pub struct VariantArm {
    /// The TypeId of the inner pattern type.
    pub type_id: std::any::TypeId,
    /// Human-readable type name for debugging.
    pub type_name: &'static str,
}

pub trait Variant: Sized + Component<Kind = kind::Variant> + Send + Sync + 'static {
    /// Number of variant arms
    const NUM_VARIANTS: usize;
    
    /// Common interface ports (macro-generated)
    const COMMON_PORTS: &'static [Port];
    
    /// Port mappings for each variant arm (macro-generated)
    const PORT_MAPPINGS: &'static [&'static [PortMap]];
    
    /// Variant arm metadata (macro-generated)
    const VARIANT_ARMS: &'static [VariantArm];
    
    /// Dependencies (macro-generated)
    const DEPENDANCIES: &'static [&'static ExecInfo];
    
    /// Schema accessor (macro generates this with OnceLock pattern)
    fn schema() -> &'static crate::session::PatternSchema {
        static SCHEMA: std::sync::OnceLock<crate::session::PatternSchema> = std::sync::OnceLock::new();
        SCHEMA.get_or_init(|| {
            let defs = Self::variant_to_defs();
            let defs_static: &'static [ColumnDef] = Box::leak(defs.into_boxed_slice());
            crate::session::PatternSchema::new(defs_static)
        })
    }
    
    /// Convert declarations to column definitions
    fn variant_to_defs() -> Vec<ColumnDef> {
        let mut defs = vec![
            ColumnDef::metadata("discriminant"),
            ColumnDef::sub::<()>("inner_ref"),
        ];
        
        defs.extend(
            Self::COMMON_PORTS.iter()
                .map(|p| ColumnDef::new(p.name, ColumnKind::Cell, false)
                    .with_direction(p.direction))
        );
        
        defs
    }

    
    /// Concatenate results from all variant arms into a unified table.
    ///
    /// This is the core operation for variants - it unions the results from
    /// each inner type, mapping their ports to the common interface and
    /// tracking which arm each row came from via the discriminant.
    fn concatenate(
        ctx: &ExecutionContext,
        dep_tables: &[&(dyn AnyTable + Send + Sync)],
    ) -> Result<Table<Self>, QueryError> {
        let schema = Self::schema();
        let mut all_entries = Vec::new();

        // Column indices (cached once)
        let discrim_idx = schema
            .index_of("discriminant")
            .ok_or_else(|| QueryError::SchemaLut("discriminant".to_string()))?;
        let inner_ref_idx = schema
            .index_of("inner_ref")
            .ok_or_else(|| QueryError::SchemaLut("inner_ref".to_string()))?;

        for (variant_idx, table) in dep_tables.iter().enumerate() {
            let port_maps = Self::PORT_MAPPINGS[variant_idx];

            for row_idx in 0..table.len() as u64 {
                let mut entry = EntryArray::with_capacity(2 + Self::COMMON_PORTS.len());

                // Initialize all entries to None
                for i in 0..(2 + Self::COMMON_PORTS.len()) {
                    entry.entries[i] = ColumnEntry::Cell { id: None };
                }

                // 1. Set discriminant
                entry.entries[discrim_idx] = ColumnEntry::Metadata {
                    id: Some(variant_idx as u64),
                };

                // 2. Set inner_ref (row index in the variant arm's table)
                entry.entries[inner_ref_idx] = ColumnEntry::Sub { id: Some(row_idx) };

                // 3. Map common ports from inner table using path resolution
                for mapping in port_maps.iter() {
                    if let Some(col_idx) = schema.index_of(mapping.common_port) {
                        let cell_id = table.resolve_path(row_idx as usize, mapping.inner, ctx);
                        entry.entries[col_idx] = ColumnEntry::Cell { id: cell_id };
                    }
                }

                all_entries.push(entry);
            }
        }

        Table::new(all_entries)
    }

    fn preload_driver(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        Self: Sized;

    fn rehydrate<'a>(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Variant> + Send + Sync + 'static;
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

    fn rehydrate<'a>(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Variant> + Send + Sync + 'static,
    {
        <T as Variant>::rehydrate(row, store, driver, key)
    }
}

#[allow(unused)]
mod test {

    use crate::{
        Wire,
        selector::Selector,
        session::ExecInfo,
        traits::{
            Netlist, Pattern,
            composite::Composite,
        },
    };

    use super::*;

    use svql_common::Dedupe;
    use svql_query::query_test;

    #[derive(Debug, Clone)]
    struct AndGate {
        a: Wire,
        b: Wire,
        y: Wire,
    }

    impl Netlist for AndGate {
        const MODULE_NAME: &'static str = "and_gate";
        const FILE_PATH: &'static str = "examples/fixtures/basic/and/verilog/and_gate.v";
        
        const PORTS: &'static [Port] = &[
            Port::input("a"),
            Port::input("b"),
            Port::output("y"),
        ];

        fn rehydrate<'a>(
            row: &Row<Self>,
            _store: &Store,
            _driver: &Driver,
            _key: &DriverKey,
        ) -> Option<Self>
        where
            Self: Component + PatternInternal<kind::Netlist> + Send + Sync + 'static,
        {
            let a_id = row.wire("a")?;
            let b_id = row.wire("b")?;
            let y_id = row.wire("y")?;

            let and_gate = AndGate {
                a: a_id,
                b: b_id,
                y: y_id,
            };

            Some(and_gate)
        }
    }

    impl Component for AndGate {
        type Kind = kind::Netlist;
    }

    #[derive(Debug, Clone)]
    pub struct And2Gates {
        and1: AndGate,
        and2: AndGate,
    }

    impl Composite for And2Gates {
        const SUBMODULES: &'static [crate::session::Submodule] = &[
            crate::session::Submodule::of::<AndGate>("and1"),
            crate::session::Submodule::of::<AndGate>("and2"),
        ];
        
        const ALIASES: &'static [crate::session::Alias] = &[
            crate::session::Alias::input("a", Selector::static_path(&["and1", "a"])),
            crate::session::Alias::input("b", Selector::static_path(&["and1", "b"])),
            crate::session::Alias::output("y", Selector::static_path(&["and2", "y"])),
        ];
        
        const CONNECTIONS: &'static [crate::traits::composite::Connection] = &[
            crate::traits::composite::Connection::new(
                Selector::static_path(&["and1", "y"]),
                Selector::static_path(&["and2", "a"])
            ),
        ];
        
        const DEPENDANCIES: &'static [&'static ExecInfo] = &[<AndGate as Pattern>::EXEC_INFO];

        fn rehydrate<'a>(
            row: &Row<Self>,
            store: &Store,
            driver: &Driver,
            key: &DriverKey,
        ) -> Option<Self>
        where
            Self: Component + PatternInternal<kind::Composite> + Send + Sync + 'static,
        {
            // Get submodule references
            let and1_ref: crate::session::Ref<AndGate> = row.sub("and1")?;
            let and2_ref: crate::session::Ref<AndGate> = row.sub("and2")?;

            // Fetch inner tables and rehydrate
            let and_table = store.get::<AndGate>()?;
            let and1_row = and_table.row(and1_ref.index())?;
            let and2_row = and_table.row(and2_ref.index())?;

            let and1 = <AndGate as Pattern>::rehydrate(&and1_row, store, driver, key)?;
            let and2 = <AndGate as Pattern>::rehydrate(&and2_row, store, driver, key)?;

            Some(And2Gates { and1, and2 })
        }

        fn preload_driver(
            driver: &Driver,
            design_key: &DriverKey,
            config: &svql_common::Config,
        ) -> Result<(), Box<dyn std::error::Error>>
        where
            Self: Sized,
        {
            <AndGate as Pattern>::preload_driver(driver, design_key, config)
        }
    }

    impl Component for And2Gates {
        type Kind = kind::Composite;
    }

    #[derive(Debug, Clone)]
    enum AndOrAnd2 {
        AndGate(AndGate),
        And2Gates(And2Gates),
    }

    impl Variant for AndOrAnd2 {
        const NUM_VARIANTS: usize = 2;
        
        const COMMON_PORTS: &'static [Port] = &[
            Port::input("a"),
            Port::input("b"),
            Port::output("y"),
        ];
        
        const PORT_MAPPINGS: &'static [&'static [PortMap]] = &[
            // Variant 0: AndGate
            &[
                PortMap::new("a", Selector::static_path(&["a"])),
                PortMap::new("b", Selector::static_path(&["b"])),
                PortMap::new("y", Selector::static_path(&["y"])),
            ],
            // Variant 1: And2Gates
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

        fn rehydrate<'a>(
            row: &Row<Self>,
            store: &Store,
            driver: &Driver,
            key: &DriverKey,
        ) -> Option<Self>
        where
            Self: Component + PatternInternal<kind::Variant> + Send + Sync + 'static,
        {
            // 1. Get discriminant
            let discrim_idx = <Self as Variant>::schema().index_of("discriminant")?;
            let discrim = row.entry_array.entries.get(discrim_idx)?.as_u64()?;

            // 2. Get inner_ref
            let inner_ref_idx = <Self as Variant>::schema().index_of("inner_ref")?;
            let inner_row_idx = row.entry_array.entries.get(inner_ref_idx)?.as_u64()?;

            // 3. Dispatch based on discriminant
            match discrim {
                0 => {
                    // AndGate
                    let inner_table = store.get::<AndGate>()?;
                    let inner_row = inner_table.row(inner_row_idx)?;
                    let inner = <AndGate as Pattern>::rehydrate(&inner_row, store, driver, key)?;
                    Some(AndOrAnd2::AndGate(inner))
                }
                1 => {
                    // And2Gates
                    let inner_table = store.get::<And2Gates>()?;
                    let inner_row = inner_table.row(inner_row_idx)?;
                    let inner = <And2Gates as Pattern>::rehydrate(&inner_row, store, driver, key)?;
                    Some(AndOrAnd2::And2Gates(inner))
                }
                _ => None,
            }
        }

        fn preload_driver(
            driver: &Driver,
            design_key: &DriverKey,
            config: &svql_common::Config,
        ) -> Result<(), Box<dyn std::error::Error>>
        where
            Self: Sized,
        {
            <AndGate as Pattern>::preload_driver(driver, design_key, config)?;
            <And2Gates as Pattern>::preload_driver(driver, design_key, config)?;
            Ok(())
        }
    }

    impl Component for AndOrAnd2 {
        type Kind = kind::Variant;
    }

    query_test!(
        name: test_and_mixed_and_tree_dedupe_none,
        query: AndOrAnd2,
        haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
        expect: 14,  // 6 AndGate + 8 And2Gates
        config: |config_builder| config_builder.dedupe(Dedupe::None)
    );

    query_test!(
        name: test_and_mixed_and_tree_dedupe_all,
        query: AndOrAnd2,
        haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
        expect: 5,  // 3 AndGate + 2 And2Gates
        config: |config_builder| config_builder.dedupe(Dedupe::All)
    );
}
