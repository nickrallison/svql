use svql_query::Search;
use svql_query::netlist::{NetlistMeta, PortDir, PortSpec};
use svql_query::queries::netlist::basic::and::and_gate::AndGate;

#[test]
fn and_netlist_meta_is_correct() {
    // Check module name and file path
    assert_eq!(<AndGate<Search> as NetlistMeta>::MODULE_NAME, "and_gate");
    assert!(
        <AndGate<Search> as NetlistMeta>::FILE_PATH
            .ends_with("examples/patterns/basic/and/and_gate.v"),
        "FILE_PATH should point to examples/patterns/basic/and/and_gate.v, got {}",
        <AndGate<Search> as NetlistMeta>::FILE_PATH
    );

    // Check ports
    let ports: &'static [PortSpec] = <AndGate<Search> as NetlistMeta>::PORTS;
    assert_eq!(ports.len(), 3, "and_gate has two inputs and one output");

    // Validate directions by name for readability
    let mut a_dir = None;
    let mut b_dir = None;
    let mut y_dir = None;

    for p in ports {
        match p.name {
            "a" => a_dir = Some(p.dir),
            "b" => b_dir = Some(p.dir),
            "y" => y_dir = Some(p.dir),
            other => panic!("unexpected port name for and_gate: {}", other),
        }
    }

    assert_eq!(a_dir, Some(PortDir::In), "a must be an input");
    assert_eq!(b_dir, Some(PortDir::In), "b must be an input");
    assert_eq!(y_dir, Some(PortDir::Out), "y must be an output");
}
