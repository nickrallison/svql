use svql_query::Search;
use svql_query::netlist::{NetlistMeta, PortDir, PortSpec};
use svql_query::queries::netlist::dff::Sdffe;

#[test]
fn sdffe_netlist_meta_is_correct() {
    // Check module name and file path
    assert_eq!(<Sdffe<Search> as NetlistMeta>::MODULE_NAME, "sdffe");
    assert!(
        <Sdffe<Search> as NetlistMeta>::FILE_PATH.ends_with("examples/patterns/basic/ff/sdffe.v"),
        "FILE_PATH should point to examples/patterns/basic/ff/sdffe.v, got {}",
        <Sdffe<Search> as NetlistMeta>::FILE_PATH
    );

    // Check ports
    let ports: &'static [PortSpec] = <Sdffe<Search> as NetlistMeta>::PORTS;
    assert_eq!(ports.len(), 4, "sdffe has 3 inputs and one output");

    // Validate directions by name for readability
    let mut clk_dir = None;
    let mut d_dir = None;
    let mut reset_dir = None;
    let mut q_dir = None;

    for p in ports {
        match p.name {
            "clk" => clk_dir = Some(p.dir),
            "d" => d_dir = Some(p.dir),
            "reset" => reset_dir = Some(p.dir),
            "q" => q_dir = Some(p.dir),
            other => panic!("unexpected port name for sdffe: {}", other),
        }
    }

    assert_eq!(clk_dir, Some(PortDir::In), "clk must be an input");
    assert_eq!(d_dir, Some(PortDir::In), "d must be an input");
    assert_eq!(reset_dir, Some(PortDir::In), "reset must be an input");
    assert_eq!(q_dir, Some(PortDir::Out), "q must be an output");
}
