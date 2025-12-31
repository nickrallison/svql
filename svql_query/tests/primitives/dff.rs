use svql_common::{Config, ModuleConfig};
use svql_driver::Driver;
use svql_query::primitives::dff::*;
use svql_query::traits::Query;
use svql_query::traits::Searchable;
use svql_query::{Instance, Search};

fn setup_driver() -> Driver {
    Driver::new_workspace().expect("Failed to create driver")
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
fn test_sdff_filtering() {
    let driver = setup_driver();
    // seq_double_sdffe.v has 2 DFFs with sync reset and NO enable
    let path = "examples/fixtures/basic/ff/verilog/seq_double_sdffe.v";
    let module = "seq_double_sdffe";

    let (key, context) = get_context::<Sdff<Search>>(&driver, path, module);

    // Should find the 2 registers as Sdff
    let matches_sdff = Sdff::<Search>::instantiate(Instance::root("q".to_string())).query(
        &driver,
        &context,
        &key,
        &Config::default(),
    );
    assert_eq!(
        matches_sdff.len(),
        2,
        "Should find 2 Sdff (Sync Reset, No En)"
    );

    // Should find 0 as Sdffe because enable is not active
    let matches_sdffe = Sdffe::<Search>::instantiate(Instance::root("q".to_string())).query(
        &driver,
        &context,
        &key,
        &Config::default(),
    );
    assert_eq!(
        matches_sdffe.len(),
        0,
        "Should find 0 Sdffe (Sync Reset + En)"
    );
}

#[test]
fn test_adff_filtering() {
    let driver = setup_driver();
    // cwe1271_fixed.v has 1 DFF with async reset and NO enable
    let path = "examples/fixtures/cwes/cwe1271/cwe1271_fixed.v";
    let module = "cwe1271_fixed";

    let (key, context) = get_context::<Adff<Search>>(&driver, path, module);

    let matches = Adff::<Search>::instantiate(Instance::root("q".to_string())).query(
        &driver,
        &context,
        &key,
        &Config::default(),
    );

    assert_eq!(matches.len(), 1, "Should find 1 Adff (Async Reset, No En)");
}

#[test]
fn test_dffe_filtering() {
    let driver = setup_driver();
    // cwe1271_en.v has 1 DFF with enable and NO reset
    let path = "examples/fixtures/cwes/cwe1271/cwe1271_en.v";
    let module = "cwe1271_en";

    let (key, context) = get_context::<Dffe<Search>>(&driver, path, module);

    let matches = Dffe::<Search>::instantiate(Instance::root("q".to_string())).query(
        &driver,
        &context,
        &key,
        &Config::default(),
    );

    assert_eq!(matches.len(), 1, "Should find 1 Dffe (No Reset, En)");
}

#[test]
fn test_adffe_filtering() {
    let driver = setup_driver();
    // cwe1234_enabled.v has:
    // 1. lock_status: async reset + enable (lock_enable)
    // 2. Data_out: async reset + enable (data_enable)
    let path = "examples/fixtures/cwes/cwe1234/cwe1234_enabled.v";
    let module = "cwe1234_enabled";

    let (key, context) = get_context::<Adffe<Search>>(&driver, path, module);

    let matches = Adffe::<Search>::instantiate(Instance::root("q".to_string())).query(
        &driver,
        &context,
        &key,
        &Config::default(),
    );

    // Note: Data_out is 16 bits, but in a flattened netlist it is 16 individual DFF cells
    // lock_status is 1 bit. Total = 17 Adffe cells.
    assert_eq!(matches.len(), 17, "Should find 17 Adffe cells");
}

#[test]
fn test_mixed_dff_types() {
    let driver = setup_driver();
    // cwe1234_mixed_resets.v has:
    // - lock_status_1: async reset, no enable (Adff)
    // - lock_status_2: sync reset (via if), no enable (Sdff)
    // - Data_out_1: async reset, enable (Adffe)
    // - Data_out_2: sync reset, enable (Sdffe)
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
