// svql_query/src/traits/recursive.rs

//! Recursive pattern traits for tree-structured matches.
//!
//! Unlike `Composite` which performs Cartesian products of independent matches,
//! `Recursive` patterns build tree structures where nodes can reference other
//! nodes of the same type (self-referential).
//!
//! # Key Differences from Composite
//!
//! | Aspect | Composite | Recursive |
//! |--------|-----------|-----------|
//! | Children | Different types | Same type (self-ref) |
//! | Search | Cartesian product + filter | Fixpoint iteration |
//! | Schema | Submodules are different types | `left_child`/`right_child` are `Ref<Self>` |
//!
//! # Example: AND Tree
//!
//! ```ignore
//! pub struct RecAnd {
//!     pub base: Ref<AndGate>,           // The AND gate at this node
//!     pub left_child: Option<Ref<RecAnd>>,  // Left subtree (None = leaf)
//!     pub right_child: Option<Ref<RecAnd>>, // Right subtree (None = leaf)
//!     pub y: Wire,                      // Output of this node
//!     pub depth: u32,                   // Tree depth (0 = leaf)
//! }
//! ```

use std::sync::OnceLock;

use crate::prelude::*;

/// Trait for recursive/tree-structured patterns.
///
/// # What It Finds
///
/// The recursive search creates **one table entry per base pattern instance**.
/// Each entry represents the **maximal tree rooted at that instance**.
///
/// For example, given this circuit:
/// ```text
///       AND0
///       /  \
///    AND1  AND2
///    / \   / \
///   a  b  c  d
/// ```
///
/// The search produces **3 `RecAnd` entries**:
/// ```text
/// RecAnd[0]: base=AND0, left_child=RecAnd[1], right_child=RecAnd[2], depth=1
///            (maximal tree rooted at AND0 - includes both children)
///
/// RecAnd[1]: base=AND1, left_child=None, right_child=None, depth=0
///            (maximal tree rooted at AND1 - just a leaf)
///
/// RecAnd[2]: base=AND2, left_child=None, right_child=None, depth=0
///            (maximal tree rooted at AND2 - just a leaf)
/// ```
///
/// # Key Properties
///
/// - **One entry per base instance**: Every AND gate gets exactly one `RecAnd` entry
/// - **Maximal subtrees**: Each entry represents the largest tree rooted at that gate
/// - **Self-referential children**: `left_child` and `right_child` point to other
///   entries in the same table (via `Ref<Self>`)
/// - **Leaves included**: Gates with no children (depth=0) are still in the table
///   because they can be children of other trees
/// - **No redundancy**: Unlike variants which union multiple tables, recursive
///   patterns produce exactly N entries for N base pattern instances
///
/// # Filtering Results
///
/// To find only "interesting" trees (non-leaves), filter by depth:
/// ```ignore
/// for row in table.rows() {
///     let rec = RecAnd::rehydrate(&row, &store, &driver, &key)?;
///     if rec.depth > 0 {
///         // This is a tree with children
///     }
/// }
/// ```
///
/// Or filter during composition by overriding `build_recursive`.
///
/// # Algorithm
///
/// 1. Get all base pattern matches (e.g., all AND gates)
/// 2. Build output→instance lookup map
/// 3. Initialize all entries as leaves (depth=0, no children)
/// 4. **Fixpoint iteration**: For each instance, check if its inputs come from
///    other instances' outputs. If so, link them as children.
/// 5. Recompute depths until convergence
///
/// # Self-Reference Handling
///
/// The schema includes `Ref<Self>` columns for children, but these are
/// **not** listed in `DEPENDANCIES` (which would cause a cycle). Instead,
/// self-references are row indices into the same table being built.
///
/// The execution system sees only the base pattern as a dependency:
/// ```text
/// EXEC_INFO.nested_dependancies = [AndGate::EXEC_INFO]
/// Schema columns = [base: Ref<AndGate>, left_child: Ref<RecAnd>, ...]
/// ```
///
/// # Implementor Responsibilities
///
/// Implementors define:
/// - `Base`: The pattern type that forms nodes (e.g., `AndGate`)
/// - `PORTS`: External interface ports (typically just output)
/// - `build_recursive`: Fixpoint algorithm to link children
/// - `recursive_rehydrate`: Reconstruct full structure from row
pub trait Recursive: Sized + Component<Kind = kind::Recursive> + Send + Sync + 'static {
    /// The base pattern type that forms nodes of the tree.
    ///
    /// For `RecAnd`, this is `AndGate`. Each node in the recursive structure
    /// wraps exactly one instance of the base pattern.
    type Base: Pattern + Component + Send + Sync + 'static;

    /// Port declarations for the recursive pattern's external interface.
    ///
    /// Typically includes at least the output port. Inputs may be implicit
    /// (derived from leaf nodes during rehydration).
    const PORTS: &'static [PortDecl];

    /// Execution dependencies (typically just the base pattern).
    ///
    /// **Important**: Do NOT include `Self` here—that would create a cycle.
    /// Self-references are handled internally during `build_recursive`.
    const DEPENDANCIES: &'static [&'static ExecInfo];

    /// Schema accessor with self-referential columns.
    ///
    /// Default schema structure:
    /// - `base`: `Ref<Self::Base>` (required)
    /// - `left_child`: `Option<Ref<Self>>` (nullable)
    /// - `right_child`: `Option<Ref<Self>>` (nullable)
    /// - Ports from `PORTS`
    /// - `depth`: metadata
    fn recursive_schema() -> &'static PatternSchema {
        static SCHEMA: OnceLock<PatternSchema> = OnceLock::new();
        SCHEMA.get_or_init(|| {
            let defs = Self::recursive_to_defs();
            let defs_static: &'static [ColumnDef] = Box::leak(defs.into_boxed_slice());
            PatternSchema::new(defs_static)
        })
    }

    /// Convert declarations to column definitions.
    ///
    /// Override this if you need a different schema structure.
    #[must_use]
    fn recursive_to_defs() -> Vec<ColumnDef> {
        let mut defs = vec![
            ColumnDef::sub::<Self::Base>("base"),
            ColumnDef::sub_nullable::<Self>("left_child"),
            ColumnDef::sub_nullable::<Self>("right_child"),
        ];

        defs.extend(
            Self::PORTS.iter().map(|p| {
                ColumnDef::new(p.name, ColumnKind::Wire, false).with_direction(p.direction)
            }),
        );

        defs.push(ColumnDef::meta("depth"));

        defs
    }

    /// Build the recursive structure using fixpoint iteration.
    ///
    /// # Algorithm
    ///
    /// 1. Get all base pattern matches from context
    /// 2. Build `output→row_index` lookup
    /// 3. Initialize all nodes as leaves (depth=0, no children)
    /// 4. Fixpoint: for each node, check if inputs come from other nodes' outputs.
    ///
    /// Executes the fixed-point iteration to discover all instances of the recursive pattern.
    ///
    /// # Errors
    ///
    /// Returns a `QueryError` if the seed search or join steps fail.
    fn build_recursive(ctx: &ExecutionContext) -> Result<Table<Self>, QueryError>;

    /// Rehydrate a row back into the concrete type.
    fn recursive_rehydrate(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
        config: &svql_common::Config,
    ) -> Option<Self>;

    /// Preload required designs into the driver.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying netlist designs cannot be loaded.
    fn preload_driver(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Create a hierarchical report node from a match row
    fn recursive_row_to_report_node(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
        _config: &svql_common::Config,
    ) -> crate::traits::display::ReportNode {
        use crate::traits::display::*;
        let config = Config::default();
        let type_name = std::any::type_name::<Self>();
        let short_name = type_name.rsplit("::").next().unwrap_or(type_name);

        let mut children = Vec::new();

        // 1. Show base pattern
        if let Some(base_row) = row
            .sub::<Self::Base>("base")
            .and_then(|base_ref| store.get::<Self::Base>().and_then(|t| t.row(base_ref)))
        {
            let mut base_node =
                Self::Base::row_to_report_node(&base_row, store, driver, key, &config);
            base_node.name = "base".to_string();
            children.push(base_node);
        }

        // 2. Show children (recursive)
        for child_name in ["left_child", "right_child"] {
            if let Some(child_row) = row
                .sub::<Self>(child_name)
                .and_then(|child_ref| store.get::<Self>().and_then(|t| t.row(child_ref)))
            {
                let mut child_node =
                    Self::recursive_row_to_report_node(&child_row, store, driver, key, &config);
                child_node.name = child_name.to_string();
                children.push(child_node);
            }
        }

        // 3. Show ports
        for port in Self::PORTS {
            if let Some(wire) = row.wire(port.name) {
                children.push(wire_to_report_node(
                    port.name,
                    wire,
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
            details: row
                .entry_array
                .entries
                .last()
                .and_then(|e| e.as_meta())
                .and_then(|m| m.as_count())
                .map(|d| format!("depth: {d}")),
            source_loc: None,
            children,
        }
    }
}

// Blanket implementation of PatternInternal for all Recursive types
impl<T> PatternInternal<kind::Recursive> for T
where
    T: Recursive + Component<Kind = kind::Recursive> + Send + Sync + 'static,
{
    const DEFS: &'static [ColumnDef] = &[]; // Unused, schema is dynamic

    // base + left_child + right_child + ports + depth
    const SCHEMA_SIZE: usize = 3 + T::PORTS.len() + 1;

    const EXEC_INFO: &'static ExecInfo = &ExecInfo {
        type_id: std::any::TypeId::of::<T>(),
        type_name: std::any::type_name::<T>(),
        search_function: |ctx| {
            search_table_any::<T>(ctx, <T as PatternInternal<kind::Recursive>>::search_table)
        },
        // Only base pattern, NOT self (would cause cycle)
        nested_dependancies: T::DEPENDANCIES,
    };

    fn internal_schema() -> &'static PatternSchema {
        T::recursive_schema()
    }

    fn preload_driver(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<(), Box<dyn std::error::Error>> {
        <T as Recursive>::preload_driver(driver, design_key, config)
    }

    fn search_table(ctx: &ExecutionContext) -> Result<Table<Self>, QueryError> {
        tracing::info!(
            "[RECURSIVE] Starting recursive search for: {}",
            std::any::type_name::<T>()
        );
        T::build_recursive(ctx)
    }

    fn internal_rehydrate(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
        config: &svql_common::Config,
    ) -> Option<Self> {
        Self::recursive_rehydrate(row, store, driver, key, config)
    }

    fn internal_row_to_report_node(
        row: &Row<Self>,
        store: &Store,
        driver: &Driver,
        key: &DriverKey,
        config: &svql_common::Config,
    ) -> crate::traits::display::ReportNode {
        Self::recursive_row_to_report_node(row, store, driver, key, config)
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use crate::define_primitive;

    use super::*;

    use svql_query::query_test;

    define_primitive!(AndGate, And, [(a, input), (b, input), (y, output)]);

    /// A node in a recursive AND tree.
    ///
    /// Each `RecAnd` entry represents the **maximal AND tree rooted at one AND gate**:
    /// - If the AND gate's inputs come from other AND gates, those are linked as `left_child`/`right_child`
    /// - If the inputs are external (not from other ANDs), the children are `None`
    /// - `depth` indicates the maximum distance to any leaf node in the subtree
    ///
    /// # Example
    ///
    /// For circuit `y = ((a & b) & (c & d))`:
    /// ```text
    ///       AND0(y=12)
    ///       /        \
    ///   AND1(10)   AND2(11)
    ///   /  \        /  \
    ///  a   b       c   d
    /// ```
    ///
    /// This produces 3 `RecAnd` entries:
    /// - `RecAnd[0]`: Maximal tree rooted at AND0 (includes AND1 and AND2 as children, depth=1)
    /// - `RecAnd[1]`: Leaf at AND1 (no AND children, depth=0)
    /// - `RecAnd[2]`: Leaf at AND2 (no AND children, depth=0)
    ///
    /// **Every AND gate appears exactly once**, representing its maximal subtree.
    #[derive(Debug, Clone)]
    pub struct RecAnd {
        /// Reference to the underlying AND gate at this node.
        pub base: Ref<AndGate>,
        /// Left subtree (None if input A is external/not from another AND).
        pub left_child: Option<Ref<Self>>,
        /// Right subtree (None if input B is external/not from another AND).
        pub right_child: Option<Ref<Self>>,
        /// Output wire of this node.
        pub y: Wire,
        /// Tree depth: 0 = leaf (no AND children), 1+ = has AND children.
        /// Represents the maximum depth of any child subtree plus 1.
        pub depth: u32,
    }

    impl Component for RecAnd {
        type Kind = kind::Recursive;
    }

    impl Recursive for RecAnd {
        type Base = AndGate;

        const PORTS: &'static [PortDecl] = &[PortDecl::output("y")];

        const DEPENDANCIES: &'static [&'static ExecInfo] = &[<AndGate as Pattern>::EXEC_INFO];

        fn recursive_schema() -> &'static PatternSchema {
            static SCHEMA: OnceLock<PatternSchema> = OnceLock::new();
            SCHEMA.get_or_init(|| {
                let defs = Self::recursive_to_defs();
                let defs_static: &'static [ColumnDef] = Box::leak(defs.into_boxed_slice());
                PatternSchema::new(defs_static)
            })
        }

        // ── RecAnd::build_recursive ──────────────────────────────────────────

        fn build_recursive(ctx: &ExecutionContext) -> Result<Table<Self>, QueryError> {
            let base_table = ctx
                .get_any_table(std::any::TypeId::of::<AndGate>())
                .ok_or_else(|| QueryError::missing_dep("AndGate"))?;

            let and_table: &Table<AndGate> = base_table
                .as_any()
                .downcast_ref()
                .ok_or_else(|| QueryError::missing_dep("AndGate (downcast failed)"))?;

            if and_table.is_empty() {
                return Table::new(vec![]);
            }

            let index = ctx.haystack_design().index();

            // Get GraphNodeIdx for each gate's output
            let nodes: Vec<GraphNodeIdx> = and_table
                .rows()
                .filter_map(|(_, row)| {
                    row.wire("y")
                        .and_then(|w| w.cell_id())
                        .and_then(|id| index.resolve_node(id))
                })
                .collect();

            if nodes.len() != and_table.len() {
                return Err(QueryError::ExecutionError(
                    "Some AndGate outputs could not be resolved to graph nodes".to_string(),
                ));
            }

            // ── FIX: use `map` instead of `filter_map` ──
            // Inputs are now Option<Wire> so rows with primary-port/constant
            // inputs are kept, preserving the 1:1 correspondence with and_table.
            struct GateInfo {
                a: Option<Wire>, // was: Wire (via filter_map)
                b: Option<Wire>, // was: Wire (via filter_map)
                y: Wire,
            }

            let gate_info: Vec<GateInfo> = and_table
                .rows()
                .map(|(_, row)| GateInfo {
                    a: row.wire("a").cloned(), // None when input is a primary port / constant
                    b: row.wire("b").cloned(),
                    y: row
                        .wire("y")
                        .expect("AndGate output 'y' must exist")
                        .clone(),
                })
                .collect();
            // gate_info[i]  ↔  and_table.row(i)  — always

            struct RecAndEntry {
                base_node: GraphNodeIdx,
                left_child: Option<GraphNodeIdx>,
                right_child: Option<GraphNodeIdx>,
                y: Wire,
                depth: u32,
            }

            let mut entries: Vec<RecAndEntry> = nodes
                .iter()
                .enumerate()
                .map(|(idx, &node)| RecAndEntry {
                    base_node: node,
                    left_child: None,
                    right_child: None,
                    y: gate_info[idx].y.clone(),
                    depth: 0,
                })
                .collect();

            let output_to_rec: HashMap<PhysicalCellId, GraphNodeIdx> = entries
                .iter()
                .filter_map(|e| e.y.cell_id().map(|id| (id, e.base_node)))
                .collect();

            let node_to_entry: HashMap<GraphNodeIdx, usize> = nodes
                .iter()
                .enumerate()
                .map(|(i, &node)| (node, i))
                .collect();

            let mut changed = true;
            let mut iterations = 0;
            const MAX_ITERATIONS: usize = 1000;

            while changed && iterations < MAX_ITERATIONS {
                changed = false;
                iterations += 1;

                for i in 0..entries.len() {
                    let _base_node = entries[i].base_node;
                    let info = &gate_info[i];

                    // ── FIX: chain through Option<Wire> ──
                    let new_left = info
                        .a
                        .as_ref()
                        .and_then(|w| w.cell_id())
                        .and_then(|id| output_to_rec.get(&id).copied());
                    let new_right = info
                        .b
                        .as_ref()
                        .and_then(|w| w.cell_id())
                        .and_then(|id| output_to_rec.get(&id).copied());

                    let children_changed =
                        new_left != entries[i].left_child || new_right != entries[i].right_child;

                    if children_changed {
                        entries[i].left_child = new_left;
                        entries[i].right_child = new_right;
                        changed = true;
                    }

                    let left_depth = new_left
                        .and_then(|node| node_to_entry.get(&node))
                        .map(|&idx| entries[idx].depth)
                        .unwrap_or(0);
                    let right_depth = new_right
                        .and_then(|node| node_to_entry.get(&node))
                        .map(|&idx| entries[idx].depth)
                        .unwrap_or(0);

                    let new_depth = if new_left.is_some() || new_right.is_some() {
                        1 + left_depth.max(right_depth)
                    } else {
                        0
                    };

                    if new_depth != entries[i].depth {
                        entries[i].depth = new_depth;
                        changed = true;
                    }
                }
            }

            if iterations >= MAX_ITERATIONS {
                tracing::warn!(
                    "RecAnd fixpoint did not converge after {} iterations",
                    MAX_ITERATIONS
                );
            }

            // EntryArray conversion — unchanged
            let schema = Self::recursive_schema();
            let base_idx = schema.index_of("base").expect("schema has 'base'");
            let left_idx = schema
                .index_of("left_child")
                .expect("schema has 'left_child'");
            let right_idx = schema
                .index_of("right_child")
                .expect("schema has 'right_child'");
            let y_idx = schema.index_of("y").expect("schema has 'y'");
            let depth_idx = schema.index_of("depth").expect("schema has 'depth'");

            let row_entries: Vec<EntryArray> = entries
                .iter()
                .enumerate()
                .map(|(entry_index, e)| {
                    let mut arr = EntryArray::with_capacity(schema.defs.len());
                    arr.set_sub_raw(base_idx, RowIndex::from_raw(entry_index as u32));
                    arr.entries[left_idx] = e
                        .left_child
                        .and_then(|node| node_to_entry.get(&node))
                        .map(|&idx| ColumnEntry::sub(RowIndex::from_raw(idx as u32)))
                        .unwrap_or(ColumnEntry::Null);
                    arr.entries[right_idx] = e
                        .right_child
                        .and_then(|node| node_to_entry.get(&node))
                        .map(|&idx| ColumnEntry::sub(RowIndex::from_raw(idx as u32)))
                        .unwrap_or(ColumnEntry::Null);
                    arr.entries[y_idx] = ColumnEntry::wire(e.y.clone());
                    arr.entries[depth_idx] = ColumnEntry::meta(MetaValue::Count(e.depth));
                    arr
                })
                .collect();

            Table::new(row_entries)
        }

        fn recursive_rehydrate(
            row: &Row<Self>,
            _store: &Store,
            _driver: &Driver,
            _key: &DriverKey,
            _config: &svql_common::Config,
        ) -> Option<Self> {
            let base: Ref<AndGate> = row.sub("base")?;
            let left_child: Option<Ref<Self>> = row.sub("left_child");
            let right_child: Option<Ref<Self>> = row.sub("right_child");
            let y = row.wire("y")?.clone();

            let schema = Self::recursive_schema();
            let depth_idx = schema.index_of("depth")?;
            let depth = match row.entry_array().entries.get(depth_idx)? {
                ColumnEntry::Meta(MetaValue::Count(c)) => Some(*c),
                _ => None,
            }?;

            Some(Self {
                base,
                left_child,
                right_child,
                y,
                depth,
            })
        }

        fn preload_driver(
            driver: &Driver,
            design_key: &DriverKey,
            config: &svql_common::Config,
        ) -> Result<(), Box<dyn std::error::Error>> {
            <AndGate as Pattern>::preload_driver(driver, design_key, config)
        }
    }

    query_test!(
        name: test_rec_and_small_tree,
        query: RecAnd,
        haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
        expect: 3
    );

    #[test]
    fn test_rec_and_depths() -> Result<(), Box<dyn std::error::Error>> {
        use svql_query::test_harness::setup_test_logging;
        setup_test_logging();

        let driver = Driver::new_workspace()?;
        let config = Config::builder().build();
        let key = DriverKey::new(
            "examples/fixtures/basic/and/verilog/small_and_tree.v",
            "small_and_tree",
        );

        let store = svql_query::run_query::<RecAnd>(&driver, &key, &config)?;
        let table = store.get::<RecAnd>().expect("Table should exist");

        // Collect depths
        let mut depths: Vec<u32> = Vec::new();
        for (_, row) in table.rows() {
            let rec =
                RecAnd::rehydrate(&row, &store, &driver, &key, &config).expect("Should rehydrate");
            depths.push(rec.depth);
        }

        depths.sort();

        // Expected: 2 leaves (depth=0), 1 root (depth=1)
        // Or depending on structure: could be depth 0, 0, 1 or 0, 1, 2
        println!("RecAnd depths: {:?}", depths);

        // At minimum, we should have some variation if there's a tree
        assert!(
            depths.iter().any(|&d| d > 0) || depths.len() <= 1,
            "Expected at least one non-leaf node in a tree with {} gates",
            depths.len()
        );

        Ok(())
    }

    #[test]
    fn test_rec_and_children_linked() -> Result<(), Box<dyn std::error::Error>> {
        use svql_query::test_harness::setup_test_logging;
        setup_test_logging();

        let driver = Driver::new_workspace()?;
        let config = Config::builder().build();
        let key = DriverKey::new(
            "examples/fixtures/basic/and/verilog/small_and_tree.v",
            "small_and_tree",
        );

        let store = svql_query::run_query::<RecAnd>(&driver, &key, &config)?;
        let table = store.get::<RecAnd>().expect("Table should exist");

        // Find a node with children
        let mut found_parent = false;
        for (_, row) in table.rows() {
            let rec =
                RecAnd::rehydrate(&row, &store, &driver, &key, &config).expect("Should rehydrate");

            if rec.left_child.is_some() || rec.right_child.is_some() {
                found_parent = true;
                println!(
                    "Found parent node: depth={}, left={:?}, right={:?}",
                    rec.depth, rec.left_child, rec.right_child
                );
            }
        }

        // In a proper tree, there should be at least one parent
        assert!(
            found_parent || table.len() <= 1,
            "Expected at least one parent node in tree"
        );

        Ok(())
    }
}
