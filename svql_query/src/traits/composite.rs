use std::marker::PhantomData;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use ahash::AHashSet;
use crate::{
    prelude::*,
    selector::Selector,
    session::join_planner::ConnectivityCache,
};

/// Connection constraint (keeping existing struct, just updating signature)
#[derive(Debug, Clone, Copy)]
pub struct Connection {
    pub from: Endpoint,
    pub to: Endpoint,
}

impl Connection {
    #[must_use]
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

/// Connections wrapper with CNF structure
#[derive(Debug, Clone, Copy)]
pub struct Connections {
    /// CNF form: outer array is AND (all groups must be satisfied),
    /// inner array is OR (at least one connection in the group must hold)
    pub connections: &'static [&'static [Connection]],
}

pub trait Composite: Sized + Component<Kind = kind::Composite> + Send + Sync + 'static {
    /// Submodule declarations (macro-generated)
    const SUBMODULES: &'static [Submodule];

    /// Port aliases (macro-generated)
    const ALIASES: &'static [Alias];

    /// Connection constraints in CNF form (macro-generated)
    const CONNECTIONS: Connections;

    /// Dependencies (macro-generated)
    const DEPENDANCIES: &'static [&'static ExecInfo];

    /// Schema accessor (macro generates this with `OnceLock` pattern)
    fn composite_schema() -> &'static crate::session::PatternSchema {
        static SCHEMA: std::sync::OnceLock<crate::session::PatternSchema> =
            std::sync::OnceLock::new();
        SCHEMA.get_or_init(|| {
            let defs = Self::composite_to_defs();
            let defs_static: &'static [ColumnDef] = Box::leak(defs.into_boxed_slice());
            crate::session::PatternSchema::new(defs_static)
        })
    }

    /// Convert declarations to column definitions
    #[must_use]
    fn composite_to_defs() -> Vec<ColumnDef> {
        let mut defs = Vec::with_capacity(Self::SUBMODULES.len() + Self::ALIASES.len());

        for sub in Self::SUBMODULES {
            defs.push(ColumnDef::new(
                sub.name,
                ColumnKind::Sub(sub.type_id),
                false,
            ));
        }

        for alias in Self::ALIASES {
            defs.push(
                ColumnDef::new(alias.port_name, ColumnKind::Cell, false)
                    .with_direction(alias.direction),
            );
        }

        defs
    }

    /// Compose submodule results into composite results
    fn compose(
        ctx: &ExecutionContext,
        dep_tables: &[&(dyn AnyTable + Send + Sync)],
    ) -> Result<Table<Self>, QueryError> {
        tracing::info!("[COMPOSITE] Starting composition for: {}", std::any::type_name::<Self>());
        
        let schema = Self::composite_schema();
        let sub_indices = &schema.submodules;
        
        tracing::debug!("[COMPOSITE] Submodule count: {}", sub_indices.len());
        for (i, &col_idx) in sub_indices.iter().enumerate() {
            let col = schema.column(col_idx);
            tracing::debug!("  [{}] {} (nullable: {}): {} rows", 
                i, col.name, col.nullable, dep_tables[i].len());
        }

        // Early exit for empty required tables
        for (i, &col_idx) in sub_indices.iter().enumerate() {
            if dep_tables[i].is_empty() && !schema.column(col_idx).nullable {
                tracing::info!("[COMPOSITE] Early exit: required submodule '{}' has no matches", 
                    schema.column(col_idx).name);
                return Table::new(vec![]);
            }
        }

        // Incremental join with filtering
        let join_order: Vec<usize> = (0..sub_indices.len()).collect();
        tracing::debug!("[COMPOSITE] Join order: {:?}", join_order);

        // Build connectivity cache once
        let connectivity_cache = ConnectivityCache::build::<Self>(dep_tables, ctx);

        let first_idx = join_order[0];
        let first_table = dep_tables[first_idx];
        tracing::info!("[COMPOSITE] Starting with {} entries from first table: {}", 
            first_table.len(), schema.column(sub_indices[first_idx]).name);

        #[cfg(feature = "parallel")]
        let mut entries: Vec<EntryArray> = if ctx.config().parallel {
            (0..first_table.len() as u32)
                .into_par_iter()
                .map(|row_idx| Self::create_partial_entry(sub_indices, first_idx, row_idx))
                .collect()
        } else {
            (0..first_table.len() as u32)
                .into_iter()
                .map(|row_idx| Self::create_partial_entry(sub_indices, first_idx, row_idx))
                .collect()
        };

        #[cfg(not(feature = "parallel"))]
        let mut entries: Vec<EntryArray> = (0..first_table.len() as u32)
            .map(|row_idx| Self::create_partial_entry(sub_indices, first_idx, row_idx))
            .collect();

        tracing::debug!("[COMPOSITE] Initial entries created: {}", entries.len());
        
        for (join_step, &join_idx) in join_order[1..].iter().enumerate() {
            let table = dep_tables[join_idx];
            let table_name = schema.column(sub_indices[join_idx]).name;
            tracing::info!("[COMPOSITE] Join step {}/{}: joining with {} ({} rows)", 
                join_step + 1, join_order.len() - 1, table_name, table.len());
            
            let before_join = entries.len();
            entries = Self::join_and_filter(
                entries,
                join_idx,
                table,
                sub_indices,
                ctx,
                &connectivity_cache,
            );
            
            tracing::debug!("[COMPOSITE] After join: {} -> {} entries ({} filtered out)", 
                before_join, entries.len(), before_join.saturating_sub(entries.len()));

            if entries.is_empty() {
                tracing::info!("[COMPOSITE] Join resulted in no matches, stopping early");
                return Table::new(vec![]);
            }
        }

        // Resolve aliases
        tracing::debug!("[COMPOSITE] Resolving {} aliases...", Self::ALIASES.len());
        let mut final_entries = Self::resolve_aliases(entries, ctx)?;
        tracing::debug!("[COMPOSITE] Aliases resolved: {} entries", final_entries.len());

        // Apply automatic deduplication
        let before_dedup = final_entries.len();
        Self::apply_deduplication(&mut final_entries);
        if before_dedup != final_entries.len() {
            tracing::debug!("[COMPOSITE] Deduplication: {} -> {} entries ({} removed)", 
                before_dedup, final_entries.len(), before_dedup - final_entries.len());
        }

        tracing::info!("[COMPOSITE] Composition complete: {} total matches", final_entries.len());
        Table::new(final_entries)
    }

    /// Apply automatic deduplication.
    fn apply_deduplication(entries: &mut Vec<EntryArray>) {
        crate::traits::apply_deduplication(entries);
    }

    /// Custom validation hook for filters (override to add filtering logic)
    fn validate_custom(_row: &Row<Self>, _ctx: &ExecutionContext) -> bool {
        true // Default: no custom filtering
    }

    /// Validate connectivity constraints
    fn validate_connections(row: &Row<Self>, ctx: &ExecutionContext) -> bool {
        // Use the cached haystack design from context instead of calling get_design
        let design = ctx.haystack_design();
        let _graph = design.index();
        
        let num_groups = Self::CONNECTIONS.connections.len();
        tracing::trace!("[COMPOSITE] Validating {} connection groups for {}", 
            num_groups, std::any::type_name::<Self>());

        // Check each CNF group (conjunction of disjunctions)
        for (group_idx, group) in Self::CONNECTIONS.connections.iter().enumerate() {
            let mut group_satisfied = false;

            // Try each alternative in this group (disjunction)
            for (conn_idx, conn) in group.iter().enumerate() {
                // SCHEMA VALIDATION (compile-time check)
                let from_valid = Row::<Self>::validate_selector_path(conn.from.selector);
                let to_valid = Row::<Self>::validate_selector_path(conn.to.selector);

                if !from_valid || !to_valid {
                    tracing::warn!(
                        "[{}] Connection group {} alternative {} has invalid selector(s)",
                        std::any::type_name::<Self>(),
                        group_idx,
                        conn_idx
                    );
                    continue; // Try next alternative
                }

                // RESOLVABILITY CHECK (runtime check - are submodules joined?)
                let from_resolvable = Self::is_selector_resolvable(row, conn.from.selector);
                let to_resolvable = Self::is_selector_resolvable(row, conn.to.selector);

                if !from_resolvable || !to_resolvable {
                    // Can't validate this connection yet - submodule not joined
                    // Optimistically assume it will be satisfied in a later join iteration
                    tracing::trace!(
                        "[{}] Connection {:?} → {:?} deferred (submodule not joined yet)",
                        std::any::type_name::<Self>(),
                        conn.from.selector.path(),
                        conn.to.selector.path()
                    );
                    group_satisfied = true; // Defer validation to later iteration
                    break;
                }

                // CONNECTIVITY VALIDATION (both sides are resolvable)
                let src_wire = row.resolve(conn.from.selector, ctx);
                let dst_wire = row.resolve(conn.to.selector, ctx);

                match (src_wire, dst_wire) {
                    (Some(s), Some(d)) if s.cell_id() == d.cell_id() => {
                        tracing::trace!(
                            "[{}] Connection {:?} → {:?} satisfied (cell {:?})",
                            std::any::type_name::<Self>(),
                            conn.from.selector.path(),
                            conn.to.selector.path(),
                            s.cell_id()
                        );
                        group_satisfied = true;
                        break; // This alternative worked
                    }
                    (None, _) | (_, None) => {
                        // Wire resolved to None despite submodule being joined
                        // Could be NULL wire (nullable port) or traversal failure
                        tracing::trace!(
                            "[{}] Connection {:?} → {:?} resolved to None (nullable port or traversal failed)",
                            std::any::type_name::<Self>(),
                            conn.from.selector.path(),
                            conn.to.selector.path()
                        );
                        continue; // Try next alternative
                    }
                    (Some(s), Some(d)) => {
                        // Different cell IDs - not connected
                        tracing::trace!(
                            "[{}] Connection {:?} → {:?} failed: different cells ({:?} != {:?})",
                            std::any::type_name::<Self>(),
                            conn.from.selector.path(),
                            conn.to.selector.path(),
                            s.cell_id(),
                            d.cell_id()
                        );
                        continue; // Try next alternative
                    }
                }
            }

            // If no alternative in this group was satisfied, validation fails
            if !group_satisfied {
                tracing::trace!(
                    "[{}] Connection group {} failed all alternatives",
                    std::any::type_name::<Self>(),
                    group_idx
                );
                return false;
            }
        }

        true
    }

        /// Check if a selector path can be fully resolved given the current row state.
    /// 
    /// Returns `false` if the path traverses through a NULL submodule reference,
    /// meaning we can't validate this connection yet (submodule not joined).
    fn is_selector_resolvable(row: &Row<Self>, selector: Selector<'_>) -> bool {
        if selector.is_empty() {
            return false;
        }

        let Some(head) = selector.head() else {
            return false;
        };

        // Single segment (e.g., ["grant"]) - just check schema
        if selector.len() == 1 {
            return Self::schema().index_of(head).is_some();
        }

        // Multi-segment (e.g., ["reg_any", "q"]) - check if submodule is populated
        let Some(col_idx) = Self::schema().index_of(head) else {
            return false;
        };

        let col_def = Self::schema().column(col_idx);
        if !col_def.kind.is_sub() {
            return false; // Not a submodule
        }

        // Check if the submodule reference is populated
        match &row.entry_array.entries[col_idx] {
            ColumnEntry::Sub { id: Some(_) } => true,  // Submodule is joined ✓
            ColumnEntry::Sub { id: None } => false,    // Submodule not joined yet
            _ => false,
        }
    }


    /// Validate (calls both connection and custom validation)
    fn validate(row: &Row<Self>, ctx: &ExecutionContext) -> bool {
        // Connection validation
        let connections_ok = Self::validate_connections(row, ctx);

        // Custom filter validation
        let custom_ok = Self::validate_custom(row, ctx);

        connections_ok && custom_ok
    }

    /// Rehydrate from row
    fn composite_rehydrate(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> Option<Self>;

    /// Create a hierarchical report node from a match row
    ///
    /// Recursive implementation uses macro-generated metadata to display
    /// submodules and alias ports.
    fn composite_row_to_report_node(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> crate::traits::display::ReportNode {
        use crate::traits::display::*;

        let config = Config::default();
        let type_name = std::any::type_name::<Self>();
        let short_name = type_name.rsplit("::").next().unwrap_or(type_name);

        let mut children = Vec::new();

        // Recursively display each submodule using metadata
        for sub in Self::SUBMODULES {
            if let Some(mut sub_node) = row
                .sub_any(sub.name)
                .and_then(|sub_ref| store.get_from_tid(sub.type_id).map(|t| (sub_ref, t)))
                .and_then(|(sub_ref, table)| table.row_to_report_node(sub_ref as usize, store, driver, key))
            {
                sub_node.name = sub.name.to_string();
                children.push(sub_node);
            }
        }

        // Display alias ports using metadata
        for alias in Self::ALIASES {
            if let Some(wire) = row.wire(alias.port_name) {
                children.push(wire_to_report_node(
                    alias.port_name,
                    &wire,
                    alias.direction,
                    driver,
                    key,
                    &config,
                ));
            }
        }

        ReportNode {
            name: short_name.to_string(),
            type_name: type_name.to_string(),
            details: None,
            source_loc: None,
            children,
        }
    }

    fn preload_driver(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        Self: Sized;

    #[must_use]
    fn create_partial_entry(sub_indices: &[usize], join_idx: usize, row_idx: u32) -> EntryArray {
        let mut entries =
            vec![ColumnEntry::Metadata { id: None }; Self::SUBMODULES.len() + Self::ALIASES.len()];
        entries[sub_indices[join_idx]] = ColumnEntry::Sub { id: Some(row_idx) };
        EntryArray::new(entries)
    }

    fn join_and_filter(
        entries: Vec<EntryArray>,
        join_idx: usize,
        table: &(dyn AnyTable + Send + Sync),
        sub_indices: &[usize],
        ctx: &ExecutionContext,
        connectivity_cache: &ConnectivityCache,
    ) -> Vec<EntryArray> {
        tracing::debug!(
            "[JOIN] Joining submodule {} ({} existing entries × {} new rows)",
            join_idx,
            entries.len(),
            table.len()
        );

        let start = std::time::Instant::now();

        #[cfg(feature = "parallel")]
        let result: Vec<EntryArray> = if ctx.config().parallel {
            entries
                .into_par_iter()
                .flat_map_iter(|entry| {
                    Self::expand_with_index(
                        entry,
                        join_idx,
                        table,
                        sub_indices,
                        ctx,
                        connectivity_cache,
                    )
                })
                .collect()
        } else {
            entries
                .into_iter()
                .flat_map(|entry| {
                    Self::expand_with_index(
                        entry,
                        join_idx,
                        table,
                        sub_indices,
                        ctx,
                        connectivity_cache,
                    )
                })
                .collect()
        };

        #[cfg(not(feature = "parallel"))]
        let result: Vec<EntryArray> = entries
            .into_iter()
            .flat_map(|entry| {
                Self::expand_with_index(
                    entry,
                    join_idx,
                    table,
                    sub_indices,
                    ctx,
                    connectivity_cache,
                )
            })
            .collect();

        tracing::debug!(
            "[JOIN] Join complete: {} candidates in {:?}",
            result.len(),
            start.elapsed()
        );

        result
    }

    /// Expand a partial assignment by joining with a new submodule.
    ///
    /// Uses connectivity index to only enumerate valid candidates.
    fn expand_with_index(
        partial_entry: EntryArray,
        new_sub_idx: usize,
        new_table: &(dyn AnyTable + Send + Sync),
        sub_indices: &[usize],
        ctx: &ExecutionContext,
        connectivity_cache: &ConnectivityCache,
    ) -> impl Iterator<Item = EntryArray> {
        let col_idx = sub_indices[new_sub_idx];

        // Determine which existing submodules the new one connects to
        let connected_subs = Self::find_connected_submodules(new_sub_idx);

        // Get valid candidates from connectivity indices
        let valid_new_rows = if connected_subs.is_empty() {
            // No constraints - try all rows
            (0..new_table.len() as u32).collect()
        } else {
            // Intersect valid sets from all connections
            Self::compute_valid_candidates(
                &partial_entry,
                new_sub_idx,
                &connected_subs,
                sub_indices,
                connectivity_cache,
                new_table.len(),
            )
        };

        // Only enumerate the valid candidates
        valid_new_rows.into_iter().filter_map(move |new_row_idx| {
            let mut candidate = partial_entry.clone();
            candidate.entries[col_idx] = ColumnEntry::Sub {
                id: Some(new_row_idx),
            };

            // Still need to validate (handles CNF OR groups, custom filters, etc.)
            let row = Row::<Self> {
                idx: 0,
                entry_array: candidate.clone(),
                _marker: PhantomData,
            };

            Self::validate(&row, ctx).then_some(candidate)
        })
    }

    /// Find which already-joined submodules the new submodule connects to.
    fn find_connected_submodules(new_sub_idx: usize) -> Vec<(usize, usize, bool)> {
        let mut result = Vec::new();

        for (cnf_idx, cnf_group) in Self::CONNECTIONS.connections.iter().enumerate() {
            for conn in cnf_group.iter() {
                let from_sub = Self::resolve_submodule(&conn.from);
                let to_sub = Self::resolve_submodule(&conn.to);

                // Check if this connection involves the new submodule
                match (from_sub, to_sub) {
                    (Some(f), Some(t)) if f == new_sub_idx => {
                        result.push((t, cnf_idx, true)); // true = new_sub is source
                    }
                    (Some(f), Some(t)) if t == new_sub_idx => {
                        result.push((f, cnf_idx, false)); // false = new_sub is target
                    }
                    _ => {}
                }
            }
        }

        result
    }

    /// Compute valid candidate rows for the new submodule.
    ///
    /// Returns intersection of all connectivity constraints.
    fn compute_valid_candidates(
        partial_entry: &EntryArray,
        new_sub_idx: usize,
        connected_subs: &[(usize, usize, bool)],
        sub_indices: &[usize],
        connectivity_cache: &ConnectivityCache,
        new_table_len: usize,
    ) -> Vec<u32> {
        let mut valid_sets: Vec<AHashSet<u32>> = Vec::new();

        for (existing_sub_idx, cnf_idx, is_new_sub_from) in connected_subs {
            // Get the row index of the already-joined submodule
            let existing_col_idx = sub_indices[*existing_sub_idx];
            let existing_row = match partial_entry.entries[existing_col_idx] {
                ColumnEntry::Sub { id: Some(row_idx) } => row_idx,
                _ => continue,
            };

            // Determine direction of the connection for the cache lookup
            let (from_sub, to_sub) = if *is_new_sub_from {
                (new_sub_idx, *existing_sub_idx)
            } else {
                (*existing_sub_idx, new_sub_idx)
            };

            // Get the connectivity index
            if let Some(index) = connectivity_cache.get(from_sub, to_sub, *cnf_idx) {
                let valid_candidates = if *is_new_sub_from {
                    // New submodule is source, existing is target
                    index.sources(existing_row)
                } else {
                    // Existing is source, new submodule is target
                    index.targets(existing_row)
                };

                if let Some(candidates) = valid_candidates {
                    valid_sets.push(candidates.clone());
                } else {
                    // No valid candidates - early exit
                    return Vec::new();
                }
            }
        }

        // Intersect all constraint sets
        if valid_sets.is_empty() {
            (0..new_table_len as u32).collect()
        } else {
            let mut result = valid_sets[0].clone();
            for other_set in &valid_sets[1..] {
                result.retain(|x| other_set.contains(x));
                if result.is_empty() {
                    return Vec::new();
                }
            }
            result.into_iter().collect()
        }
    }

    fn resolve_submodule(selector: &Endpoint) -> Option<usize> {
        let head = selector.selector.head()?;
        Self::composite_schema().submodule_index(head)
    }

    fn resolve_aliases(
        entries: Vec<EntryArray>,
        ctx: &ExecutionContext,
    ) -> Result<Vec<EntryArray>, QueryError> {
        #[cfg(feature = "parallel")]
        if ctx.config().parallel {
            let final_entries = entries
                .into_par_iter()
                .map(|mut entry| {
                    let row = Row::<Self> {
                        idx: 0,
                        entry_array: entry.clone(),
                        _marker: PhantomData,
                    };

                    for alias in Self::ALIASES {
                        let wire_ref = row
                            .resolve(alias.target, ctx)
                            .and_then(|w| w.cell_id())
                            .map(crate::wire::WireRef::Cell);

                        if let Some(idx) = Self::composite_schema().index_of(alias.port_name) {
                            entry.entries[idx] = ColumnEntry::Wire { value: wire_ref };
                        }
                    }

                    entry
                })
                .collect();

            return Ok(final_entries);
        }

        let final_entries = entries
            .into_iter()
            .map(|mut entry| {
                let row = Row::<Self> {
                    idx: 0,
                    entry_array: entry.clone(),
                    _marker: PhantomData,
                };

                for alias in Self::ALIASES {
                    let wire_ref = row
                        .resolve(alias.target, ctx)
                        .and_then(|w| w.cell_id())
                        .map(crate::wire::WireRef::Cell);

                    if let Some(idx) = Self::composite_schema().index_of(alias.port_name) {
                        entry.entries[idx] = ColumnEntry::Wire { value: wire_ref };
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

    fn internal_schema() -> &'static crate::session::PatternSchema {
        T::composite_schema()
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

        for sub_idx in &T::composite_schema().submodules {
            let tid = T::composite_schema()
                .column(*sub_idx)
                .as_submodule()
                .expect("Idx should point to submodule");
            let table = ctx.get_any_table(tid).ok_or_else(|| {
                QueryError::MissingDependency(format!(
                    "TypeId {:?}, Col: {}",
                    tid,
                    T::composite_schema().column(*sub_idx).name
                ))
            })?;
            dep_tables.push(table);
        }

        T::compose(ctx, &dep_tables)
    }

    fn internal_rehydrate<'a>(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Composite> + Send + Sync + 'static,
    {
        Self::composite_rehydrate(row, store, driver, key)
    }

    fn internal_row_to_report_node(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
    ) -> crate::traits::display::ReportNode {
        Self::composite_row_to_report_node(row, store, driver, key)
    }
}

#[allow(unused)]
pub(crate) mod test {

    use crate::{
        Wire,
        traits::{Netlist, Pattern},
    };

    use super::{
        Alias, Component, Composite, Connection, Connections, Driver, DriverKey, ExecInfo, Row,
        Selector, Store, Submodule, kind,
    };

    use crate::traits::netlist::test::AndGate;

    use svql_query::query_test;

    #[derive(Debug, Clone, Composite)]
    #[or_to(from = ["and1", "y"], to = [["and2", "a"], ["and2", "b"]])]
    #[filter(|_row, _ctx| { 
        true
    })]
    pub struct And2Gates {
        #[submodule]
        pub and1: AndGate,
        #[submodule]
        pub and2: AndGate,
        #[alias(input, target = ["and1", "a"])]
        pub a: Wire,
        #[alias(input, target = ["and1", "b"])]
        pub b: Wire,
        #[alias(output, target = ["and2", "y"])]
        pub y: Wire,
    }

    query_test!(
        name: test_and2gates_small_and_tree,
        query: And2Gates,
        haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
        expect: 2  // Automatically deduplicated
    );

    #[derive(Debug, Clone)]
    pub struct ManualAnd2Gates {
        pub and1: AndGate,
        pub and2: AndGate,
        pub a: Wire,
        pub b: Wire,
        pub y: Wire,
    }

    impl Component for ManualAnd2Gates {
        type Kind = kind::Composite;
    }

    impl Composite for ManualAnd2Gates {
        const SUBMODULES: &'static [Submodule] = &[
            Submodule::of::<AndGate>("and1"),
            Submodule::of::<AndGate>("and2"),
        ];

        const ALIASES: &'static [Alias] = &[
            Alias::input("a", Selector::static_path(&["and1", "a"])),
            Alias::input("b", Selector::static_path(&["and1", "b"])),
            Alias::output("y", Selector::static_path(&["and2", "y"])),
        ];

        const CONNECTIONS: Connections = Connections {
            connections: &[&[
                Connection::new(
                    Selector::static_path(&["and1", "y"]),
                    Selector::static_path(&["and2", "a"]),
                ),
                Connection::new(
                    Selector::static_path(&["and1", "y"]),
                    Selector::static_path(&["and2", "b"]),
                ),
            ]],
        };

        const DEPENDANCIES: &'static [&'static ExecInfo] = &[<AndGate as Pattern>::EXEC_INFO];

        fn composite_rehydrate(
            row: &Row<Self>,
            store: &Store,
            driver: &Driver,
            key: &DriverKey,
        ) -> Option<Self> {
            let and1 = {
                let sub_ref = row.sub::<AndGate>("and1")?;
                let sub_table = store.get::<AndGate>()?;
                let sub_row = sub_table.row(sub_ref.index())?;
                <AndGate as Pattern>::rehydrate(&sub_row, store, driver, key)?
            };

            let and2 = {
                let sub_ref = row.sub::<AndGate>("and2")?;
                let sub_table = store.get::<AndGate>()?;
                let sub_row = sub_table.row(sub_ref.index())?;
                <AndGate as Pattern>::rehydrate(&sub_row, store, driver, key)?
            };

            Some(Self {
                and1,
                and2,
                a: row.wire("a")?,
                b: row.wire("b")?,
                y: row.wire("y")?,
            })
        }

        fn preload_driver(
            driver: &Driver,
            design_key: &DriverKey,
            config: &svql_common::Config,
        ) -> Result<(), Box<dyn std::error::Error>> {
            <AndGate as Pattern>::preload_driver(driver, design_key, config)?;
            Ok(())
        }
    }

    query_test!(
        name: test_manual_and2gates_small_tree,
        query: ManualAnd2Gates,
        haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
        expect: 2
    );
}
