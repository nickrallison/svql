use svql_query::prelude::*;

fn setup_driver() -> Driver {
    Driver::new_workspace().expect("Failed to create driver")
}

fn get_context<'a, Q: Pattern>(driver: &Driver, path: &str, module: &str) -> (DriverKey, Context) {
    let config = ModuleConfig::default().with_flatten(true);
    let (key, design) = driver
        .get_or_load_design(path, module, &config)
        .expect("Failed to load design");

    let context = Q::context(driver, &config)
        .unwrap()
        .with_design(key.clone(), design);

    (key, context)
}

#[test]
fn test_and_gate_scan() {
    let driver = setup_driver();
    let (key, context) = get_context::<AndGate<Search>>(
        &driver,
        "examples/fixtures/basic/and/verilog/small_and_tree.v",
        "small_and_tree",
    );

    let query = AndGate::<Search>::instantiate(Instance::root("q".to_string()));
    let matches = query.execute(&driver, &context, &key, &Config::default());

    // small_and_tree has: (a & b) & (c & d) -> 3 AND gates
    assert_eq!(matches.len(), 3, "Should find exactly 3 AND gates");
}

#[test]
fn test_dff_gate_scan() {
    let driver = setup_driver();
    let (key, context) = get_context::<DffAny<Search>>(
        &driver,
        "examples/fixtures/basic/ff/verilog/seq_double_sdffe.v",
        "seq_double_sdffe",
    );

    let query = DffAny::<Search>::instantiate(Instance::root("q".to_string()));
    let matches = query.execute(&driver, &context, &key, &Config::default());

    // seq_double_sdffe has two always @(posedge clk) blocks
    assert_eq!(matches.len(), 2, "Should find exactly 2 DFFs");
}

#[test]
fn test_mixed_gates_scan() {
    let driver = setup_driver();
    let path = "examples/fixtures/composite/logic_tree/mixed_gates.v";
    let module = "mixed_gates";

    // Test AND
    let (key, ctx) = get_context::<AndGate<Search>>(&driver, path, module);
    let matches = AndGate::<Search>::instantiate(Instance::root("q".to_string())).execute(
        &driver,
        &ctx,
        &key,
        &Config::default(),
    );
    assert_eq!(matches.len(), 1, "Should find 1 AND gate");

    // Test XOR
    let (key, ctx) = get_context::<XorGate<Search>>(&driver, path, module);
    let matches = XorGate::<Search>::instantiate(Instance::root("q".to_string())).execute(
        &driver,
        &ctx,
        &key,
        &Config::default(),
    );
    assert_eq!(matches.len(), 1, "Should find 1 XOR gate");

    // Test NOT
    let (key, ctx) = get_context::<NotGate<Search>>(&driver, path, module);
    let matches = NotGate::<Search>::instantiate(Instance::root("q".to_string())).execute(
        &driver,
        &ctx,
        &key,
        &Config::default(),
    );
    assert_eq!(matches.len(), 1, "Should find 1 NOT gate");

    // Test OR
    // assign y = and_out | xor_out | not_out;
    // This usually flattens to two 2-input OR gates
    let (key, ctx) = get_context::<OrGate<Search>>(&driver, path, module);
    let matches = OrGate::<Search>::instantiate(Instance::root("q".to_string())).execute(
        &driver,
        &ctx,
        &key,
        &Config::default(),
    );
    assert_eq!(matches.len(), 2, "Should find 2 OR gates");
}

#[test]
fn test_mux_gate_scan() {
    let driver = setup_driver();
    let (key, context) = get_context::<MuxGate<Search>>(
        &driver,
        "examples/fixtures/composite/logic_tree/mux_tree.v",
        "mux_tree",
    );

    let query = MuxGate::<Search>::instantiate(Instance::root("q".to_string()));
    let matches = query.execute(&driver, &context, &key, &Config::default());

    // mux_tree has 3 ternary operators (?)
    assert_eq!(matches.len(), 3, "Should find exactly 3 MUXes");
}

#[test]
fn test_xor_chain_scan() {
    let driver = setup_driver();
    let (key, context) = get_context::<XorGate<Search>>(
        &driver,
        "examples/fixtures/composite/logic_tree/xor_chain.v",
        "xor_chain",
    );

    let query = XorGate::<Search>::instantiate(Instance::root("q".to_string()));
    let matches = query.execute(&driver, &context, &key, &Config::default());

    // xor_chain has 3 XOR operations (^)
    assert_eq!(matches.len(), 3, "Should find exactly 3 XOR gates");
}

#[test]
fn test_eq_gate_scan() {
    let driver = setup_driver();
    // grant_access.v: assign grant = (usr_id == correct_id) ? 1'b1 : 1'b0;
    let (key, context) = get_context::<EqGate<Search>>(
        &driver,
        "examples/patterns/security/access_control/grant_access/verilog/grant_access.v",
        "grant_access",
    );

    let query = EqGate::<Search>::instantiate(Instance::root("q".to_string()));
    let matches = query.execute(&driver, &context, &key, &Config::default());

    // Should find exactly 1 equality comparison cell
    assert_eq!(matches.len(), 1, "Should find 1 Eq gate in grant_access");
}

#[test]
fn test_complex_comparison_scan() {
    let driver = setup_driver();
    // cwe1280_fixed.v contains: grant_access = (usr_id == 3'h4) ? 1'b1 : 1'b0;
    // This involves an Eq gate and a Mux gate.
    let path = "examples/fixtures/cwes/cwe1280/verilog/cwe1280_fixed.v";
    let module = "cwe1280_fixed";

    // Check Eq
    let (key_eq, ctx_eq) = get_context::<EqGate<Search>>(&driver, path, module);
    let matches_eq = EqGate::<Search>::instantiate(Instance::root("q".to_string())).execute(
        &driver,
        &ctx_eq,
        &key_eq,
        &Config::default(),
    );
    assert_eq!(
        matches_eq.len(),
        1,
        "Should find 1 Eq gate in cwe1280_fixed"
    );

    // Check Mux
    let (key_mux, ctx_mux) = get_context::<MuxGate<Search>>(&driver, path, module);
    let matches_mux = MuxGate::<Search>::instantiate(Instance::root("q".to_string())).execute(
        &driver,
        &ctx_mux,
        &key_mux,
        &Config::default(),
    );

    // One mux for the grant_access assignment,
    // and another for: data_out = (grant_access) ? data_in : data_out;
    assert_eq!(
        matches_mux.len(),
        2,
        "Should find 2 Mux gates in cwe1280_fixed"
    );
}
