#![allow(missing_docs)]
#![allow(clippy::missing_docs_in_private_items)]

use prjunnamed_netlist::Design;
use svql_common::{GraphIndex, GraphNodeIdx, ModuleConfig, YosysModule};

mod common;

fn create_test_design() -> Design {
    // Use an existing fixture to create a design
    let module = YosysModule::new(
        "examples/fixtures/basic/and/verilog/small_and_tree.v",
        "small_and_tree",
    )
    .expect("Failed to create YosysModule for fixture");

    module
        .import_design(&ModuleConfig::default())
        .expect("Failed to import design")
}

#[test]
fn test_graph_index_build() {
    let design = create_test_design();
    let index = GraphIndex::build(&design);

    // Verify basic properties
    assert!(index.num_cells() > 0);
}

#[test]
fn test_graph_index_fanin_fanout_symmetry() {
    let design = create_test_design();
    let index = GraphIndex::build(&design);

    // For each cell, verify fan-in/fan-out symmetry
    for i in 0..index.num_cells() {
        let cell_idx = GraphNodeIdx::new(i as u32);

        // Check that if A is in B's fanout, then B is in A's fanin
        for fanout in index.fanout_set(cell_idx) {
            assert!(index.fanin_set(*fanout).contains(&cell_idx));
        }

        for fanin in index.fanin_set(cell_idx) {
            assert!(index.fanout_set(*fanin).contains(&cell_idx));
        }
    }
}

#[test]
fn test_graph_index_physical_resolution() {
    let design = create_test_design();
    let index = GraphIndex::build(&design);

    // Verify that physical ID resolution is bijective
    for i in 0..index.num_cells() {
        let node_idx = GraphNodeIdx::new(i as u32);
        let physical = index.resolve_physical(node_idx);
        let resolved = index.resolve_node(physical);

        assert_eq!(resolved, Some(node_idx));
    }
}

#[test]
fn test_graph_index_io_mapping() {
    let design = create_test_design();
    let index = GraphIndex::build(&design);

    // Verify I/O mappings exist
    let input_map = index.get_input_fanout_by_name_indices();
    let output_map = index.get_output_fanin_by_name_indices();

    // Check for some known ports in the fixture
    assert!(input_map.contains_key("a"));
    assert!(input_map.contains_key("e")); // Even if unused, it should be in the map
    assert!(output_map.contains_key("y"));

    // Outputs should generally have fanin if the design is correct
    for (name, fanin) in output_map {
        assert!(!fanin.is_empty(), "Output {} has no fanin", name);
    }
}
