use svql_query::prelude::*;
use tracing::{Level, info};

// use svql_query_lib::security::primitives::locked_register::LockedRegister;

#[derive(Debug, Clone, Netlist)]
#[netlist(
    file = "examples/fixtures/basic/and/verilog/and_gate.v",
    module = "and_gate"
)]
#[allow(dead_code)]
pub(crate) struct AndGate {
    #[port(input)]
    pub a: Wire,
    #[port(input)]
    pub b: Wire,
    #[port(output)]
    pub y: Wire,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let driver = Driver::new_workspace()?;
    // let mut driver = Driver::new_workspace_yosys(
    //     "/home/nick/Applications/tabby-linux-x64-latest/tabby/bin/yosys",
    // )?;

    let args = std::env::args().collect::<Vec<String>>();

    let design_path: &str = args.get(1).map_or(
        "examples/fixtures/larger_designs/json/hummingbirdv2/e203_soc_netlist.json",
        std::string::String::as_str,
    );

    let design_module: &str = args
        .get(2)
        .map_or("e203_soc_top", std::string::String::as_str);

    let max_recursion_depth: usize = args
        .get(3)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(2);

    let use_raw_import: bool = args
        .get(4)
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(false);

    let haystack_options = ModuleConfig {
        // default
        load_raw: use_raw_import,
        ..Default::default()
    };

    let config = Config::builder()
        .match_length(MatchLength::NeedleSubsetHaystack)
        .dedupe(Dedupe::All)
        .haystack_options(haystack_options)
        .max_recursion_depth(Some(max_recursion_depth))
        .build();

    let design_key = DriverKey::new(design_path, design_module);
    info!("Loading design...");

    let _design_result = driver.get_design(&design_key, &config.haystack_options);

    // Test the DataFrame API
    info!("Executing query with DataFrame API...");
    let store = svql_query::run_query::<AndGate>(&driver, &design_key, &config)?;

    println!("\n=== DataFrame API Results ===");
    println!("{store}");

    for (_, table) in store.tables() {
        println!("\n=== Table Details ===");
        println!("{table}");
    }

    // if let Some(table) = store.get::<LockedRegister<Search>>() {
    //     println!("\n=== LockedRegister Table Details ===");
    //     println!("{}", table);
    // }

    // if let Some(table) = store.get::<AsyncDffMuxEnable<Search>>() {
    //     println!("\n=== AsyncDffMuxEnable Table Details ===");
    //     println!("{}", table);
    // }

    Ok(())
}
