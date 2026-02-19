//! svql_query_lib/src/security/cdc.rs
//! Patterns for detecting Clock Domain Crossing (CDC) violations.

use std::sync::OnceLock;

use crate::primitives::{AdcGate, AndGate, MulGate, MuxGate, NotGate, OrGate, ULtGate, XorGate};
use svql_query::prelude::*;

/// A variant matching any standard combinational logic gate.
/// Used as the base for the recursive logic cone.
#[allow(missing_docs)]
#[derive(Debug, Clone, Variant)]
#[variant_ports(input(a), input(b), input(c), output(y))]
pub enum AnyLogicGate {
    #[map(a = ["a"], b = ["b"], y = ["y"])]
    And(AndGate),
    #[map(a = ["a"], b = ["b"], y = ["y"])]
    Or(OrGate),
    #[map(a = ["a"], b = ["b"], y = ["y"])]
    Xor(XorGate),
    #[map(a = ["a"], y = ["y"])]
    Not(NotGate),
    #[map(a = ["a"], b = ["b"], c = ["sel"], y = ["y"])]
    Mux(MuxGate),
    #[map(a = ["a"], b = ["b"], y = ["y"])]
    ULt(ULtGate),
    #[map(a = ["a"], b = ["b"], c = ["cin"], y = ["y"])]
    Adc(AdcGate),
    #[map(a = ["a"], b = ["b"], y = ["y"])]
    Mul(MulGate),
}

/// A recursive tree of combinational logic gates.
///
/// Supports gates with up to 3 inputs (e.g., Mux, Adc).
/// Inputs driven by other logic gates become children.
/// Inputs driven by non-logic (FF, Const, Port) become leaf inputs.
#[derive(Debug, Clone)]
pub struct LogicCone {
    /// The gate at this node.
    pub base: Ref<AnyLogicGate>,
    /// Children driven by other logic gates.
    /// child_0, child_1, child_2.
    pub children: [Option<Ref<Self>>; 3],
    /// Output of this node.
    pub y: Wire,
    /// Depth of this node in the tree.
    pub depth: u32,
    /// All leaf input wires (inputs NOT from other logic gates).
    pub leaf_inputs: Vec<Wire>,
}

impl Component for LogicCone {
    type Kind = kind::Recursive;
}

impl Recursive for LogicCone {
    type Base = AnyLogicGate;

    const PORTS: &'static [PortDecl] = &[PortDecl::output("y")];
    const DEPENDANCIES: &'static [&'static ExecInfo] = &[<AnyLogicGate as Pattern>::EXEC_INFO];

    fn recursive_to_defs() -> Vec<ColumnDef> {
        vec![
            ColumnDef::sub::<Self::Base>("base"),
            ColumnDef::sub_nullable::<Self>("child_0"),
            ColumnDef::sub_nullable::<Self>("child_1"),
            ColumnDef::sub_nullable::<Self>("child_2"),
            ColumnDef::output("y"),
            ColumnDef::meta("depth"),
            ColumnDef::wire_array("leaf_inputs"),
        ]
    }

    fn recursive_schema() -> &'static PatternSchema {
        static SCHEMA: OnceLock<PatternSchema> = OnceLock::new();
        SCHEMA.get_or_init(|| {
            let defs = Self::recursive_to_defs();
            let defs_static: &'static [ColumnDef] = Box::leak(defs.into_boxed_slice());
            PatternSchema::new(defs_static)
        })
    }

    fn build_recursive(ctx: &ExecutionContext) -> Result<Table<Self>, QueryError> {
        let base_table = ctx
            .get_any_table(std::any::TypeId::of::<AnyLogicGate>())
            .ok_or_else(|| QueryError::missing_dep("AnyLogicGate"))?;

        let logic_table: &Table<AnyLogicGate> = base_table
            .as_any()
            .downcast_ref()
            .ok_or_else(|| QueryError::missing_dep("AnyLogicGate (downcast)"))?;

        if logic_table.is_empty() {
            return Table::new(vec![]);
        }

        let index = ctx.haystack_design().index();

        // Resolve GraphNodeIdx for each gate's output
        let nodes: Vec<GraphNodeIdx> = logic_table
            .rows()
            .filter_map(|(_, row)| {
                row.wire("y")
                    .and_then(|w| w.cell_id())
                    .and_then(|id| index.resolve_node(id))
            })
            .collect();

        if nodes.len() != logic_table.len() {
            return Err(QueryError::ExecutionError(
                "Some AnyLogicGate outputs could not be resolved".to_string(),
            ));
        }

        // Internal entry struct
        struct LogicEntry {
            base_node: GraphNodeIdx,
            children: [Option<GraphNodeIdx>; 3],
            y: Wire,
            depth: u32,
            leaf_inputs: Vec<Wire>,
        }

        let mut entries: Vec<LogicEntry> = nodes
            .iter()
            .enumerate()
            .map(|(idx, &node)| {
                let y_wire = logic_table
                    .row_at(idx as u32)
                    .unwrap()
                    .wire("y")
                    .unwrap()
                    .clone();
                LogicEntry {
                    base_node: node,
                    children: [None, None, None],
                    y: y_wire,
                    depth: 0,
                    leaf_inputs: Vec::new(),
                }
            })
            .collect();

        // Lookup maps
        let node_to_entry: HashMap<GraphNodeIdx, usize> = nodes
            .iter()
            .enumerate()
            .map(|(i, &node)| (node, i))
            .collect();

        // Fixpoint iteration
        let mut changed = true;
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 1000;

        while changed && iterations < MAX_ITERATIONS {
            changed = false;
            iterations += 1;

            for i in 0..entries.len() {
                let node = entries[i].base_node;
                let fanin = index.fanin_set(node);

                let mut new_children_vec = Vec::new();
                let mut new_leaf_inputs = Vec::new();

                // Iterate all fan-in
                for pred_node in fanin {
                    if let Some(&child_entry_idx) = node_to_entry.get(pred_node) {
                        // It is a logic gate -> it's a child
                        new_children_vec.push(*pred_node);
                        // Inherit leaf inputs
                        new_leaf_inputs.extend(entries[child_entry_idx].leaf_inputs.clone());
                    } else {
                        // It is NOT a logic gate -> leaf input
                        let pred_wrapper = index.get_cell_by_index(*pred_node);
                        let wire = Wire::from(pred_wrapper.debug_index());
                        new_leaf_inputs.push(wire);
                    }
                }

                // Map children to fixed slots (max 3)
                let new_children: [Option<GraphNodeIdx>; 3] = [
                    new_children_vec.first().copied(),
                    new_children_vec.get(1).copied(),
                    new_children_vec.get(2).copied(),
                ];

                // Check for changes
                if new_children != entries[i].children {
                    entries[i].children = new_children;
                    changed = true;
                }

                // Deduplicate leaf inputs
                new_leaf_inputs.sort_by_key(|w| w.cell_id().map(|id| id.storage_key()));
                new_leaf_inputs.dedup_by_key(|w| w.cell_id().map(|id| id.storage_key()));

                if new_leaf_inputs != entries[i].leaf_inputs {
                    entries[i].leaf_inputs = new_leaf_inputs;
                    changed = true;
                }

                // Update depth
                let max_child_depth = new_children
                    .iter()
                    .flatten()
                    .filter_map(|n| node_to_entry.get(n).map(|&idx| entries[idx].depth))
                    .max()
                    .unwrap_or(0);

                let new_depth = if new_children.iter().any(Option::is_some) {
                    1 + max_child_depth
                } else {
                    0
                };

                if new_depth != entries[i].depth {
                    entries[i].depth = new_depth;
                    changed = true;
                }
            }
        }

        // Convert to EntryArray
        let schema = Self::recursive_schema();
        let base_idx = schema.index_of("base").unwrap();
        let c0_idx = schema.index_of("child_0").unwrap();
        let c1_idx = schema.index_of("child_1").unwrap();
        let c2_idx = schema.index_of("child_2").unwrap();
        let y_idx = schema.index_of("y").unwrap();
        let depth_idx = schema.index_of("depth").unwrap();
        let leaf_idx = schema.index_of("leaf_inputs").unwrap();

        let row_entries: Vec<EntryArray> = entries
            .iter()
            .enumerate()
            .map(|(entry_index, e)| {
                let mut arr = EntryArray::with_capacity(schema.defs.len());
                arr.set_sub_raw(base_idx, RowIndex::from_u32(entry_index as u32));

                // Helper to map node to entry index
                let resolve_child = |n: Option<GraphNodeIdx>| -> ColumnEntry {
                    n.and_then(|node| node_to_entry.get(&node))
                        .map(|&idx| ColumnEntry::Sub(RowIndex::from_u32(idx as u32)))
                        .unwrap_or(ColumnEntry::Null)
                };

                arr.entries[c0_idx] = resolve_child(e.children[0]);
                arr.entries[c1_idx] = resolve_child(e.children[1]);
                arr.entries[c2_idx] = resolve_child(e.children[2]);

                arr.entries[y_idx] = ColumnEntry::Wire(e.y.clone());
                arr.entries[depth_idx] = ColumnEntry::meta(MetaValue::Count(e.depth));
                arr.entries[leaf_idx] = ColumnEntry::WireArray(e.leaf_inputs.clone());
                arr
            })
            .collect();

        Table::new(row_entries)
    }

    fn recursive_rehydrate(
        row: &Row<Self>,
        _: &Store,
        _: &Driver,
        _: &DriverKey,
        _: &svql_common::Config,
    ) -> Option<Self> {
        Some(Self {
            base: row.sub("base")?,
            children: [row.sub("child_0"), row.sub("child_1"), row.sub("child_2")],
            y: row.wire("y")?.clone(),
            depth: row
                .entry_array()
                .entries
                .get(Self::recursive_schema().index_of("depth")?)?
                .as_meta()?
                .as_count()?,
            leaf_inputs: row
                .wire_bundle("leaf_inputs")
                .map(|s| s.to_vec())
                .unwrap_or_default(),
        })
    }

    fn preload_driver(
        driver: &Driver,
        key: &DriverKey,
        cfg: &svql_common::Config,
    ) -> Result<(), Box<dyn std::error::Error>> {
        <AnyLogicGate as Pattern>::preload_driver(driver, key, cfg)
    }
}

impl LogicCone {
    /// Computes the total size of this logic cone (number of nodes).
    ///
    /// # Panics
    ///
    /// Panics if a child entry cannot be resolved from the store, or if
    /// recursive rehydration fails.
    pub fn size(self, store: &Store, driver: &Driver, key: &DriverKey) -> usize {
        1 + self
            .children
            .iter()
            .flatten()
            .map(|c| {
                let row = store
                    .resolve::<Self>(*c)
                    .expect("Child entry should be resolvable");
                let rehydrate = Self::recursive_rehydrate(
                    &row,
                    store,
                    driver,
                    key,
                    &svql_common::Config::default(),
                )
                .expect("Rehydration should succeed");
                rehydrate.size(store, driver, key)
            })
            .sum::<usize>()
    }
}
