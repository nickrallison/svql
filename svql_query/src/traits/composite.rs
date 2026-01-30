use svql_driver::{Driver, DriverKey};
use std::marker::PhantomData;

use crate::{
    prelude::{ColumnDef, ColumnKind, QueryError, Table},
    selector::Selector,
    session::{Alias, AnyTable, ColumnEntry, EntryArray, ExecInfo, ExecutionContext, Port, Row, Store, Submodule},
    traits::{Component, PatternInternal, kind, search_table_any},
};

/// Connection constraint (keeping existing struct, just updating signature)
#[derive(Debug, Clone, Copy)]
pub struct Connection {
    pub from: Endpoint,
    pub to: Endpoint,
}

impl Connection {
    pub const fn new(from: Selector<'static>, to: Selector<'static>) -> Self {
        Self {
            from: Endpoint { selector: from },
            to: Endpoint { selector: to },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Endpoint {
    pub selector: Selector<'static>,
}

pub trait Composite: Sized + Component<Kind = kind::Composite> + Send + Sync + 'static {
    /// Submodule declarations (macro-generated)
    const SUBMODULES: &'static [Submodule];
    
    /// Port aliases (macro-generated)
    const ALIASES: &'static [Alias];
    
    /// Connection constraints (macro-generated)
    const CONNECTIONS: &'static [Connection];
    
    /// Dependencies (macro-generated)
    const DEPENDANCIES: &'static [&'static ExecInfo];
    
    /// Schema accessor (macro generates this with OnceLock pattern)
    fn schema() -> &'static crate::session::PatternSchema {
        static SCHEMA: std::sync::OnceLock<crate::session::PatternSchema> = std::sync::OnceLock::new();
        SCHEMA.get_or_init(|| {
            let defs = Self::composite_to_defs();
            let defs_static: &'static [ColumnDef] = Box::leak(defs.into_boxed_slice());
            crate::session::PatternSchema::new(defs_static)
        })
    }
    
    /// Convert declarations to column definitions
    fn composite_to_defs() -> Vec<ColumnDef> {
        let mut defs = Vec::with_capacity(Self::SUBMODULES.len() + Self::ALIASES.len());
        
        for sub in Self::SUBMODULES {
            defs.push(ColumnDef::new(
                sub.name, 
                ColumnKind::Sub(sub.type_id), 
                false
            ));
        }
        
        for alias in Self::ALIASES {
            defs.push(ColumnDef::new(
                alias.port_name,
                ColumnKind::Cell,
                false
            ).with_direction(alias.direction));
        }
        
        defs
    }

    
    /// Compose submodule results into composite results
    fn compose(
        ctx: &ExecutionContext,
        dep_tables: &[&(dyn AnyTable + Send + Sync)],
    ) -> Result<Table<Self>, QueryError> {
        let schema = Self::schema();
        let sub_indices = &schema.submodules;
        
        // Early exit for empty required tables
        for (i, &col_idx) in sub_indices.iter().enumerate() {
            if dep_tables[i].is_empty() && !schema.column(col_idx).nullable {
                return Table::new(vec![]);
            }
        }
        
        // Incremental join with filtering
        let join_order: Vec<usize> = (0..sub_indices.len()).collect();
        
        let first_idx = join_order[0];
        let first_table = dep_tables[first_idx];
        let mut entries: Vec<EntryArray> = (0..first_table.len() as u64)
            .map(|row_idx| Self::create_partial_entry(sub_indices, first_idx, row_idx))
            .collect();
        
        for &join_idx in &join_order[1..] {
            let table = dep_tables[join_idx];
            entries = Self::join_and_filter(entries, join_idx, table, sub_indices, dep_tables, ctx);
            
            if entries.is_empty() {
                return Table::new(vec![]);
            }
        }
        
        // Resolve aliases
        let final_entries = Self::resolve_aliases(entries, dep_tables, ctx)?;
        
        Table::new(final_entries)
    }
    
    /// Validate connectivity constraints
    fn validate(
        row: &Row<Self>,
        _dep_tables: &[&(dyn AnyTable + Send + Sync)],
        ctx: &ExecutionContext,
    ) -> bool {
        let driver = ctx.driver();
        let key = ctx.design_key();
        
        let design = match driver.get_design(&key, &ctx.config().haystack_options) {
            Ok(d) => d,
            Err(_) => return false,
        };
        let _graph = design.index();
        
        // Check each connection constraint
        for conn in Self::CONNECTIONS {
            let src_wire = row.resolve(conn.from.selector, ctx);
            let dst_wire = row.resolve(conn.to.selector, ctx);
            
            match (src_wire, dst_wire) {
                (Some(s), Some(d)) if s.id() == d.id() => continue,
                (Some(_), Some(_)) => return false,
                (None, _) | (_, None) => return false,
            }
        }
        
        true
    }
    
    /// Rehydrate from row (macro generates submodule lookups)
    fn rehydrate(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> Option<Self>;
    
    fn preload_driver(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        Self: Sized;
    
    // --- Helper methods (stay in trait for debuggability) ---
    
    fn create_partial_entry(
        sub_indices: &[usize],
        join_idx: usize,
        row_idx: u64,
    ) -> EntryArray {
        let mut entries = vec![ColumnEntry::Metadata { id: None }; Self::SUBMODULES.len() + Self::ALIASES.len()];
        entries[sub_indices[join_idx]] = ColumnEntry::Sub { id: Some(row_idx) };
        EntryArray::new(entries)
    }
    
    fn join_and_filter(
        entries: Vec<EntryArray>,
        join_idx: usize,
        table: &(dyn AnyTable + Send + Sync),
        sub_indices: &[usize],
        dep_tables: &[&(dyn AnyTable + Send + Sync)],
        ctx: &ExecutionContext,
    ) -> Vec<EntryArray> {
        let col_idx = sub_indices[join_idx];
        
        entries.into_iter()
            .flat_map(|entry| {
                (0..table.len() as u64).filter_map(move |new_row_idx| {
                    let mut candidate = entry.clone();
                    candidate.entries[col_idx] = ColumnEntry::Sub { id: Some(new_row_idx) };
                    
                    let row = Row::<Self> {
                        idx: 0,
                        entry_array: candidate.clone(),
                        _marker: PhantomData,
                    };
                    
                    Self::validate(&row, dep_tables, ctx).then_some(candidate)
                })
            })
            .collect()
    }
    
    fn resolve_aliases(
        entries: Vec<EntryArray>,
        _dep_tables: &[&(dyn AnyTable + Send + Sync)],
        ctx: &ExecutionContext,
    ) -> Result<Vec<EntryArray>, QueryError> {
        let final_entries = entries.into_iter()
            .map(|mut entry| {
                let row = Row::<Self> {
                    idx: 0,
                    entry_array: entry.clone(),
                    _marker: PhantomData,
                };
                
                for alias in Self::ALIASES {
                    let cell_id = row.resolve(alias.target, ctx)
                        .map(|w| w.id());
                    
                    if let Some(idx) = Self::schema().index_of(alias.port_name) {
                        entry.entries[idx] = ColumnEntry::Cell { id: cell_id };
                    }
                }
                
                entry
            })
            .collect();
        
        Ok(final_entries)
    }
}

impl<T> PatternInternal<kind::Composite> for T
where
    T: Composite + Component<Kind = kind::Composite> + Send + Sync + 'static,
{
    const DEFS: &'static [ColumnDef] = &[]; // Placeholder, not used anymore
    
    const SCHEMA_SIZE: usize = T::SUBMODULES.len() + T::ALIASES.len();
    
    const EXEC_INFO: &'static crate::session::ExecInfo = &crate::session::ExecInfo {
        type_id: std::any::TypeId::of::<T>(),
        type_name: std::any::type_name::<T>(),
        search_function: |ctx| {
            search_table_any::<T>(ctx, <T as PatternInternal<kind::Composite>>::search_table)
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
        <T as Composite>::preload_driver(driver, design_key, config)
    }
    
    fn search_table(ctx: &ExecutionContext) -> Result<Table<Self>, QueryError>
    where
        Self: Send + Sync + 'static,
    {
        let mut dep_tables = Vec::new();
        
        for sub_idx in T::schema().submodules.iter() {
            let tid = T::schema()
                .column(*sub_idx)
                .as_submodule()
                .expect("Idx should point to submodule");
            let table = ctx.get_any_table(tid).ok_or_else(|| {
                QueryError::MissingDependency(format!(
                    "TypeId {:?}, Col: {}",
                    tid,
                    T::schema().column(*sub_idx).name
                ))
            })?;
            dep_tables.push(table);
        }
        
        T::compose(ctx, &dep_tables)
    }
    
    fn rehydrate<'a>(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Composite> + Send + Sync + 'static,
    {
        <T as Composite>::rehydrate(row, store, driver, key)
    }
}

#[allow(unused)]
mod test {

    use crate::{
        Wire,
        traits::{Netlist, Pattern},
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
        const SUBMODULES: &'static [Submodule] = &[
            Submodule::of::<AndGate>("and1"),
            Submodule::of::<AndGate>("and2"),
        ];
        
        const ALIASES: &'static [Alias] = &[
            Alias::input("a", Selector::static_path(&["and1", "a"])),
            Alias::input("b", Selector::static_path(&["and1", "b"])),
            Alias::output("y", Selector::static_path(&["and2", "y"])),
        ];
        
        const CONNECTIONS: &'static [Connection] = &[
            Connection::new(
                Selector::static_path(&["and1", "y"]),
                Selector::static_path(&["and2", "a"])
            ),
        ];
        
        const DEPENDANCIES: &'static [&'static ExecInfo] = &[<AndGate as Pattern>::EXEC_INFO];

        fn rehydrate(
            row: &Row<Self>,
            store: &Store,
            driver: &Driver,
            key: &DriverKey,
        ) -> Option<Self> {
            let and1_ref = row.sub::<AndGate>("and1")?;
            let and2_ref = row.sub::<AndGate>("and2")?;
            
            let and_table = store.get::<AndGate>()?;
            let and1 = <AndGate as Netlist>::rehydrate(&and_table.row(and1_ref.index())?, store, driver, key)?;
            let and2 = <AndGate as Netlist>::rehydrate(&and_table.row(and2_ref.index())?, store, driver, key)?;
            
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

    query_test!(
        name: test_and2gates_small_and_tree_dedupe_none,
        query: And2Gates,
        haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
        expect: 8,
        config: |config_builder| config_builder.dedupe(Dedupe::None)
    );

    query_test!(
        name: test_and2gates_small_and_tree_dedupe_all,
        query: And2Gates,
        haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
        expect: 2,
        config: |config_builder| config_builder.dedupe(Dedupe::All)
    );
}
