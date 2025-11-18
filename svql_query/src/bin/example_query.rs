// svql_query/src/bin/example_query.rs
use svql_common::{Config, Dedupe, MatchLength};
use svql_driver::Driver;
use svql_query::instance::Instance;
use svql_query::security::cwe1234::Cwe1234;
use svql_query::traits::composite::SearchableComposite;
use tracing::{debug, info};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    let cfg = Config::builder()
        .match_length(MatchLength::First)
        .dedupe(Dedupe::None)
        .max_recursion_depth(Some(4))
        .build();

    let driver = Driver::new_workspace()?;

    // let design_path = "examples/fixtures/cwes/cwe1234/cwe1234_not_alternating.v";
    // let design_module = "cwe1234_not_alternating";
    // let (design_key, design_arc) =
    //     driver.get_or_load_design(design_path, design_module, &cfg.haystack_options)?;

    let design_path = "/home/nick/Downloads/hackatdac21/generated/openpiton_tile_flat.json";
    // let design_path = "/Users/nick/Downloads/openpiton_tile_flat.json";
    let design_module = "tile";
    let (design_key, design_arc) = driver.get_or_load_design_raw(design_path, design_module)?;

    info!("Loading design from {}:{}", design_path, design_module);
    let cells = design_arc.design().iter_cells().count();
    info!("Design has {} cells", cells);

    info!("Design loaded with key: {:#?}", design_key);

    let context = Cwe1234::context(&driver, &cfg.haystack_options)?;
    let context = context.with_design(design_key.clone(), design_arc.clone());

    let index = design_arc.index();

    let time_start = std::time::Instant::now();
    debug!("Starting query at {:?}", time_start);
    let cwe1234_results = Cwe1234::query(
        &design_key,
        &context,
        Instance::root("cwe1234".to_string()),
        &cfg,
    );

    let count = cwe1234_results.len();
    println!("Found {} Matches", count);

    for result in cwe1234_results.iter() {
        let locked_reg = &result.locked_register;

        let data_out = &locked_reg
            .data_out()
            .val
            .as_ref()
            .unwrap()
            .design_node_ref
            .as_ref()
            .unwrap();

        let unlock_ors = &result.unlock_logic.rec_or;
        let ors = unlock_ors.fanin_set(index);

        for o in ors.iter() {
            println!("Unlock OR Node: {:#?}", o);
        }

        println!("{:#?}", data_out);
    }

    let time_end = std::time::Instant::now();
    debug!("Ending query at {:?}", time_end);

    let duration = time_end.duration_since(time_start);
    info!("Query completed in {:?}", duration);
    Ok(())
}

// fn main() {}
