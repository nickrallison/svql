use svql_common::{Config, ModuleConfig};
use svql_driver::Driver;
use svql_query::primitives::dff::*;
use svql_query::traits::Query;
use svql_query::traits::Searchable;
use svql_query::{Instance, Search};

fn setup_driver() -> Driver {
    // Driver::new_workspace().expect("Failed to create driver")
    Driver::new_workspace_yosys("/home/nick/Applications/tabby-linux-x64-latest/tabby/bin/yosys")
        .expect("Failed to create driver")
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
    let path = "examples/patterns/basic/ff/rtlil/sdffe.il";
    let module = "sdffe";

    let (key, context) = get_context::<Sdffe<Search>>(&driver, path, module);
    let query = Sdffe::<Search>::instantiate(Instance::root("q".to_string()));
    let matches = query.query(&driver, &context, &key, &Config::default());

    assert_eq!(matches.len(), 1, "Should find 1 Sdffe (Sync Reset + En)");
}

#[test]
fn test_adffe_primitive() {
    let driver = setup_driver();
    let path = "examples/patterns/basic/ff/rtlil/adffe.il";
    let module = "adffe";

    let (key, context) = get_context::<Adffe<Search>>(&driver, path, module);
    let query = Adffe::<Search>::instantiate(Instance::root("q".to_string()));
    let matches = query.query(&driver, &context, &key, &Config::default());

    assert_eq!(matches.len(), 1, "Should find 1 Adffe (Async Reset + En)");
}

#[test]
fn test_sdff_primitive() {
    let driver = setup_driver();
    let path = "examples/patterns/basic/ff/rtlil/sdff.il";
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
    let path = "examples/patterns/basic/ff/rtlil/adff.il";
    let module = "adff";

    let (key, context) = get_context::<Adff<Search>>(&driver, path, module);
    let query = Adff::<Search>::instantiate(Instance::root("q".to_string()));
    let matches = query.query(&driver, &context, &key, &Config::default());

    assert_eq!(matches.len(), 1, "Should find 1 Adff (Async Reset, No En)");
}

#[test]
fn test_dffe_primitive() {
    let driver = setup_driver();
    let path = "examples/patterns/basic/ff/rtlil/dffe.il";
    let module = "dffe";

    let (key, context) = get_context::<Dffe<Search>>(&driver, path, module);
    let query = Dffe::<Search>::instantiate(Instance::root("q".to_string()));
    let matches = query.query(&driver, &context, &key, &Config::default());

    assert_eq!(matches.len(), 1, "Should find 1 Dffe (No Reset, En)");
}

#[test]
fn test_dff_any_primitive() {
    let driver = setup_driver();
    let path = "examples/patterns/basic/ff/rtlil/dff.il";
    let module = "dff";

    let (key, context) = get_context::<DffAny<Search>>(&driver, path, module);
    let query = DffAny::<Search>::instantiate(Instance::root("q".to_string()));
    let matches = query.query(&driver, &context, &key, &Config::default());

    assert_eq!(matches.len(), 1, "Should find 1 DffAny (Basic DFF)");
}
