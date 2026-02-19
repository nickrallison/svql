//! Implementations of recursive tree patterns for standard gates.

use std::sync::OnceLock;

use crate::primitives::gates::*;
use svql_query::prelude::*;

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
    /// All leaf input wires of this subtree (inputs that are NOT from other AND gates).
    /// Used for set-based connectivity checking with `#[connect_any]`.
    pub leaf_inputs: Vec<Wire>,
}

impl Component for RecAnd {
    type Kind = kind::Recursive;
}

impl Recursive for RecAnd {
    type Base = AndGate;

    const PORTS: &'static [PortDecl] = &[PortDecl::output("y")];

    const DEPENDANCIES: &'static [&'static ExecInfo] = &[<AndGate as Pattern>::EXEC_INFO];

    /// Override schema to include leaf_inputs column
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
        defs.push(ColumnDef::wire_array("leaf_inputs"));

        defs
    }

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
            /// Input wire 'a', None if it's a primary port or constant
            a: Option<Wire>, // was: Wire (via filter_map)
            /// Input wire 'b', None if it's a primary port or constant
            b: Option<Wire>, // was: Wire (via filter_map)
            /// Output wire 'y'
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
            /// The base graph node index
            base_node: GraphNodeIdx,
            /// Left child node index, if any
            left_child: Option<GraphNodeIdx>,
            /// Right child node index, if any
            right_child: Option<GraphNodeIdx>,
            /// Output wire 'y'
            y: Wire,
            /// Depth in the recursive structure
            depth: u32,
            /// Leaf input wires (inputs not from other AND gates)
            leaf_inputs: Vec<Wire>,
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
                leaf_inputs: Vec::new(),
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

                // Compute leaf_inputs: collect inputs that are NOT from other AND gates
                let mut new_leaf_inputs = Vec::new();

                // Check left input
                if let Some(left_node) = new_left {
                    // Has left child - inherit its leaf inputs
                    if let Some(&left_idx) = node_to_entry.get(&left_node) {
                        new_leaf_inputs.extend(entries[left_idx].leaf_inputs.clone());
                    }
                } else {
                    // No left child - this is a leaf input
                    if let Some(a) = &info.a {
                        new_leaf_inputs.push(a.clone());
                    }
                }

                // Check right input
                if let Some(right_node) = new_right {
                    // Has right child - inherit its leaf inputs
                    if let Some(&right_idx) = node_to_entry.get(&right_node) {
                        new_leaf_inputs.extend(entries[right_idx].leaf_inputs.clone());
                    }
                } else {
                    // No right child - this is a leaf input
                    if let Some(b) = &info.b {
                        new_leaf_inputs.push(b.clone());
                    }
                }

                // Update if changed
                if new_leaf_inputs != entries[i].leaf_inputs {
                    entries[i].leaf_inputs = new_leaf_inputs;
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
        let leaf_inputs_idx = schema
            .index_of("leaf_inputs")
            .expect("schema has 'leaf_inputs'");

        let row_entries: Vec<EntryArray> = entries
            .iter()
            .enumerate()
            .map(|(entry_index, e)| {
                let mut arr = EntryArray::with_capacity(schema.defs.len());
                arr.set_sub_raw(base_idx, RowIndex::from_u32(entry_index as u32));
                arr.entries[left_idx] = e
                    .left_child
                    .and_then(|node| node_to_entry.get(&node))
                    .map(|&idx| ColumnEntry::Sub(RowIndex::from_u32(idx as u32)))
                    .unwrap_or(ColumnEntry::Null);
                arr.entries[right_idx] = e
                    .right_child
                    .and_then(|node| node_to_entry.get(&node))
                    .map(|&idx| ColumnEntry::Sub(RowIndex::from_u32(idx as u32)))
                    .unwrap_or(ColumnEntry::Null);
                arr.entries[y_idx] = ColumnEntry::Wire(e.y.clone());
                arr.entries[depth_idx] = ColumnEntry::meta(MetaValue::Count(e.depth));
                // Store leaf_inputs as WireArray
                arr.entries[leaf_inputs_idx] = ColumnEntry::WireArray(e.leaf_inputs.clone());
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
        let depth = row
            .entry_array()
            .entries
            .get(depth_idx)?
            .as_meta()?
            .as_count()?;

        // Get leaf_inputs from WireArray column
        let leaf_inputs = row
            .wire_bundle("leaf_inputs")
            .map(|s| s.to_vec())
            .unwrap_or_default();

        Some(Self {
            base,
            left_child,
            right_child,
            y,
            depth,
            leaf_inputs,
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

/// A recursive tree formed of boolean OR gates.
#[derive(Debug, Clone)]
pub struct RecOr {
    /// Reference to the underlying AND gate at this node.
    pub base: Ref<OrGate>,
    /// Left subtree (None if input A is external/not from another AND).
    pub left_child: Option<Ref<Self>>,
    /// Right subtree (None if input B is external/not from another AND).
    pub right_child: Option<Ref<Self>>,
    /// Output wire of this node.
    pub y: Wire,
    /// Tree depth: 0 = leaf (no AND children), 1+ = has AND children.
    /// Represents the maximum depth of any child subtree plus 1.
    pub depth: u32,
    /// All leaf input wires of this subtree (inputs that are NOT from other OR gates).
    /// Used for set-based connectivity checking with `#[connect_any]`.
    pub leaf_inputs: Vec<Wire>,
}

impl Component for RecOr {
    type Kind = kind::Recursive;
}

impl Recursive for RecOr {
    type Base = OrGate;

    const PORTS: &'static [PortDecl] = &[PortDecl::output("y")];

    const DEPENDANCIES: &'static [&'static ExecInfo] = &[<OrGate as Pattern>::EXEC_INFO];

    /// Override schema to include leaf_inputs column
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
        defs.push(ColumnDef::wire_array("leaf_inputs"));

        defs
    }

    fn recursive_schema() -> &'static PatternSchema {
        static SCHEMA: OnceLock<PatternSchema> = OnceLock::new();
        SCHEMA.get_or_init(|| {
            let defs = Self::recursive_to_defs();
            let defs_static: &'static [ColumnDef] = Box::leak(defs.into_boxed_slice());
            PatternSchema::new(defs_static)
        })
    }

    // ── RecOr::build_recursive ──────────────────────────────────────────

    fn build_recursive(ctx: &ExecutionContext) -> Result<Table<Self>, QueryError> {
        let base_table = ctx
            .get_any_table(std::any::TypeId::of::<OrGate>())
            .ok_or_else(|| QueryError::missing_dep("OrGate"))?;

        let or_table: &Table<OrGate> = base_table
            .as_any()
            .downcast_ref()
            .ok_or_else(|| QueryError::missing_dep("OrGate (downcast failed)"))?;

        if or_table.is_empty() {
            return Table::new(vec![]);
        }

        let index = ctx.haystack_design().index();

        // Get GraphNodeIdx for each gate's output
        let nodes: Vec<GraphNodeIdx> = or_table
            .rows()
            .filter_map(|(_, row)| {
                row.wire("y")
                    .and_then(|w| w.cell_id())
                    .and_then(|id| index.resolve_node(id))
            })
            .collect();

        if nodes.len() != or_table.len() {
            return Err(QueryError::ExecutionError(
                "Some OrGate outputs could not be resolved to graph nodes".to_string(),
            ));
        }

        // ── FIX: use `map` instead of `filter_map` ──
        // Inputs are now Option<Wire> so rows with primary-port/constant
        // inputs are kept, preserving the 1:1 correspondence with or_table.
        struct GateInfo {
            /// Input wire 'a', None if it's a primary port or constant
            a: Option<Wire>, // was: Wire (via filter_map)
            /// Input wire 'b', None if it's a primary port or constant
            b: Option<Wire>, // was: Wire (via filter_map)
            /// Output wire 'y'
            y: Wire,
        }

        let gate_info: Vec<GateInfo> = or_table
            .rows()
            .map(|(_, row)| GateInfo {
                a: row.wire("a").cloned(), // None when input is a primary port / constant
                b: row.wire("b").cloned(),
                y: row.wire("y").expect("OrGate output 'y' must exist").clone(),
            })
            .collect();
        // gate_info[i]  ↔  or_table.row(i)  — always

        struct RecOrEntry {
            /// The base graph node index
            base_node: GraphNodeIdx,
            /// Left child node index, if any
            left_child: Option<GraphNodeIdx>,
            /// Right child node index, if any
            right_child: Option<GraphNodeIdx>,
            /// Output wire 'y'
            y: Wire,
            /// Depth in the recursive structure
            depth: u32,
            /// Leaf input wires (inputs not from other OR gates)
            leaf_inputs: Vec<Wire>,
        }

        let mut entries: Vec<RecOrEntry> = nodes
            .iter()
            .enumerate()
            .map(|(idx, &node)| RecOrEntry {
                base_node: node,
                left_child: None,
                right_child: None,
                y: gate_info[idx].y.clone(),
                depth: 0,
                leaf_inputs: Vec::new(),
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

                // Compute leaf_inputs: collect inputs that are NOT from other OR gates
                let mut new_leaf_inputs = Vec::new();

                // Check left input
                if let Some(left_node) = new_left {
                    // Has left child - inherit its leaf inputs
                    if let Some(&left_idx) = node_to_entry.get(&left_node) {
                        new_leaf_inputs.extend(entries[left_idx].leaf_inputs.clone());
                    }
                } else {
                    // No left child - this is a leaf input
                    if let Some(a) = &info.a {
                        new_leaf_inputs.push(a.clone());
                    }
                }

                // Check right input
                if let Some(right_node) = new_right {
                    // Has right child - inherit its leaf inputs
                    if let Some(&right_idx) = node_to_entry.get(&right_node) {
                        new_leaf_inputs.extend(entries[right_idx].leaf_inputs.clone());
                    }
                } else {
                    // No right child - this is a leaf input
                    if let Some(b) = &info.b {
                        new_leaf_inputs.push(b.clone());
                    }
                }

                // Update if changed
                if new_leaf_inputs != entries[i].leaf_inputs {
                    entries[i].leaf_inputs = new_leaf_inputs;
                    changed = true;
                }
            }
        }

        if iterations >= MAX_ITERATIONS {
            tracing::warn!(
                "RecOr fixpoint did not converge after {} iterations",
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
        let leaf_inputs_idx = schema
            .index_of("leaf_inputs")
            .expect("schema has 'leaf_inputs'");

        let row_entries: Vec<EntryArray> = entries
            .iter()
            .enumerate()
            .map(|(entry_index, e)| {
                let mut arr = EntryArray::with_capacity(schema.defs.len());
                arr.set_sub_raw(base_idx, RowIndex::from_u32(entry_index as u32));
                arr.entries[left_idx] = e
                    .left_child
                    .and_then(|node| node_to_entry.get(&node))
                    .map(|&idx| ColumnEntry::Sub(RowIndex::from_u32(idx as u32)))
                    .unwrap_or(ColumnEntry::Null);
                arr.entries[right_idx] = e
                    .right_child
                    .and_then(|node| node_to_entry.get(&node))
                    .map(|&idx| ColumnEntry::Sub(RowIndex::from_u32(idx as u32)))
                    .unwrap_or(ColumnEntry::Null);
                arr.entries[y_idx] = ColumnEntry::Wire(e.y.clone());
                arr.entries[depth_idx] = ColumnEntry::meta(MetaValue::Count(e.depth));
                // Store leaf_inputs as WireArray
                arr.entries[leaf_inputs_idx] = ColumnEntry::WireArray(e.leaf_inputs.clone());
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
        let base: Ref<OrGate> = row.sub("base")?;
        let left_child: Option<Ref<Self>> = row.sub("left_child");
        let right_child: Option<Ref<Self>> = row.sub("right_child");
        let y = row.wire("y")?.clone();

        let schema = Self::recursive_schema();
        let depth_idx = schema.index_of("depth")?;
        let depth = row
            .entry_array()
            .entries
            .get(depth_idx)?
            .as_meta()?
            .as_count()?;

        // Get leaf_inputs from WireArray column
        let leaf_inputs = row
            .wire_bundle("leaf_inputs")
            .map(|s| s.to_vec())
            .unwrap_or_default();

        Some(Self {
            base,
            left_child,
            right_child,
            y,
            depth,
            leaf_inputs,
        })
    }

    fn preload_driver(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<(), Box<dyn std::error::Error>> {
        <OrGate as Pattern>::preload_driver(driver, design_key, config)
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use svql_query::prelude::*;

    use super::*;
    use svql_query::query_test;

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
