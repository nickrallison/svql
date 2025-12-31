use svql_common::{Config, ModuleConfig};
use svql_driver::Driver;
use svql_query::primitives::dff::*;
use svql_query::traits::Query;
use svql_query::traits::Searchable;
use svql_query::{Instance, Search};

fn setup_driver() -> Driver {
    // Driver::new_workspace().expect("Failed to create driver")
    Driver::new_workspace_yosys(
        "/home/nick/Applications/tabby-linux-x64-latest/tabby/bin/yosys",
    ).expect("Failed to create driver")
}

fn get_context<'a, Q: Query>(
    driver: &Driver,
    path: &str,
    module: &str,
) -> (::svql_driver::DriverKey, ::svql_driver::Context) {
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
fn test_sdffe_primitive() {
    let driver = setup_driver();
    let path = "examples/patterns/basic/ff/verilog/sdffe.v";
    let module = "sdffe";

    let (key, context) = get_context::<Sdffe<Search>>(&driver, path, module);
    let query = Sdffe::<Search>::instantiate(Instance::root("q".to_string()));
    let matches = query.query(&driver, &context, &key, &Config::default());

    assert_eq!(matches.len(), 1, "Should find 1 Sdffe (Sync Reset + En)");
}

#[test]
fn test_adffe_primitive() {
    let driver = setup_driver();
    let path = "examples/patterns/basic/ff/verilog/adffe.v";
    let module = "adffe";

    let (key, context) = get_context::<Adffe<Search>>(&driver, path, module);
    let query = Adffe::<Search>::instantiate(Instance::root("q".to_string()));
    let matches = query.query(&driver, &context, &key, &Config::default());

    assert_eq!(matches.len(), 1, "Should find 1 Adffe (Async Reset + En)");
}

#[test]
fn test_sdff_primitive() {
    let driver = setup_driver();
    let path = "examples/patterns/basic/ff/verilog/sdff.v";
    let module = "sdff";

    let (key, context) = get_context::<Sdff<Search>>(&driver, path, module);
    let query = Sdff::<Search>::instantiate(Instance::root("q".to_string()));
    let matches = query.query(&driver, &context, &key, &Config::default());

    assert_eq!(matches.len(), 1, "Should find 1 Sdff (Sync Reset, No En)");

    // Negative test: Sdffe should not match because enable is inactive
    let matches_sdffe = Sdffe::<Search>::instantiate(Instance::root("q".to_string())).query(
        &driver,
        &context,
        &key,
        &Config::default(),
    );
    assert_eq!(
        matches_sdffe.len(),
        0,
        "Sdffe should not match Sdff fixture"
    );
}

#[test]
fn test_adff_primitive() {
    let driver = setup_driver();
    let path = "examples/patterns/basic/ff/verilog/adff.v";
    let module = "adff";

    let (key, context) = get_context::<Adff<Search>>(&driver, path, module);
    let query = Adff::<Search>::instantiate(Instance::root("q".to_string()));
    let matches = query.query(&driver, &context, &key, &Config::default());

    assert_eq!(matches.len(), 1, "Should find 1 Adff (Async Reset, No En)");
}

#[test]
fn test_dffe_primitive() {
    let driver = setup_driver();
    let path = "examples/patterns/basic/ff/verilog/dffe.v";
    let module = "dffe";

    let (key, context) = get_context::<Dffe<Search>>(&driver, path, module);
    let query = Dffe::<Search>::instantiate(Instance::root("q".to_string()));
    let matches = query.query(&driver, &context, &key, &Config::default());

    assert_eq!(matches.len(), 1, "Should find 1 Dffe (No Reset, En)");
}

#[test]
fn test_dff_any_primitive() {
    let driver = setup_driver();
    let path = "examples/patterns/basic/ff/verilog/dff.v";
    let module = "dff";

    let (key, context) = get_context::<DffAny<Search>>(&driver, path, module);
    let query = DffAny::<Search>::instantiate(Instance::root("q".to_string()));
    let matches = query.query(&driver, &context, &key, &Config::default());

    assert_eq!(matches.len(), 1, "Should find 1 DffAny (Basic DFF)");
}

#[test]
fn test_mixed_dff_types() {
    let driver = setup_driver();
    let path = "examples/fixtures/cwes/cwe1234/cwe1234_mixed_resets.v";
    let module = "cwe1234_mixed_resets";

    let config = Config::default();

    // Check Adff (lock_status_1)
    let (key, ctx) = get_context::<Adff<Search>>(&driver, path, module);
    let m_adff = Adff::<Search>::instantiate(Instance::root("q".to_string()))
        .query(&driver, &ctx, &key, &config);
    assert_eq!(m_adff.len(), 1, "Should find 1 Adff");

    // Check Sdff (lock_status_2)
    let m_sdff = Sdff::<Search>::instantiate(Instance::root("q".to_string()))
        .query(&driver, &ctx, &key, &config);
    assert_eq!(m_sdff.len(), 1, "Should find 1 Sdff");

    // Check Adffe (Data_out_1 is 16 bits)
    let m_adffe = Adffe::<Search>::instantiate(Instance::root("q".to_string()))
        .query(&driver, &ctx, &key, &config);
    assert_eq!(m_adffe.len(), 16, "Should find 16 Adffe cells");

    // Check Sdffe (Data_out_2 is 16 bits)
    let m_sdffe = Sdffe::<Search>::instantiate(Instance::root("q".to_string()))
        .query(&driver, &ctx, &key, &config);
    assert_eq!(m_sdffe.len(), 16, "Should find 16 Sdffe cells");
}
