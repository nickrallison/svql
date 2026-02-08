#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use svql_query::prelude::*;
use svql_query_lib::{
    DffAny,
    security::primitives::locked_register::AsyncDffMuxEnable,
    security::{cwe1234::Cwe1234, primitives::locked_register::LockedRegister},
};
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_thread_ids(true)
        .init();

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

    let use_raw_import: bool = args
        .get(3)
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(false);

    let parallel: bool = args
        .get(4)
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(true);

    let haystack_options = ModuleConfig {
        // default
        load_raw: use_raw_import,
        ..Default::default()
    };

    let config = Config::builder()
        .match_length(MatchLength::NeedleSubsetHaystack)
        .haystack_options(haystack_options)
        .parallel(parallel)
        .build();

    let design_key = DriverKey::new(design_path, design_module);
    info!("Loading design...: {:?}", design_key);

    // let _design_result = driver.get_design(&design_key, &config.haystack_options);

    // Test the DataFrame API
    info!("Executing query with DataFrame API...");
    // let store = svql_query::run_query::<Cwe1234>(&driver, &design_key, &config)?;

    // Check if we should use AsyncDffMuxEnable for locked_reg haystack

    let store = svql_query::run_query::<Cwe1234>(&driver, &design_key, &config)?;

    println!("\n=== DataFrame API Results ===");
    println!("{store}");

    for (_, table) in store.tables() {
        println!("\n=== Table Details ===");
        println!("{table}");
    }

    for row in store
        .get::<Cwe1234>()
        .expect("Store should have table")
        .rows()
        .take(20)
    {
        let report = row.render(&store, &driver, &design_key);
        // if report.contains("i_ar_arbiter.gen_arbiter.gen_int_rr.rr_d") {
        println!("{}", report);
        // }
    }

    Ok(())
}
