// svql_query/src/bin/example_query.rs
use svql_common::{Config, Dedupe, MatchLength};
use svql_driver::Driver;
use svql_query::composite::SearchableComposite;
use svql_query::queries::composite::rec_and::RecAnd;
use svql_query::{
    composite::SearchableEnumComposite, instance::Instance,
    queries::enum_composite::and_any::AndAny,
};
use tracing::{debug, info};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    let cfg = Config::builder()
        .match_length(MatchLength::First)
        .dedupe(Dedupe::None)
        .build();

    let design_path = "examples/patterns/basic/and/verilog/and_2_seq.v";
    let design_module = "and_2_seq";

    info!("Loading design from {}:{}", design_path, design_module);
    let driver = Driver::new_workspace()?;
    // let (design_key, design_arc) = driver.get_or_load_design_raw(design_path, design_module)?;
    let (design_key, design_arc) =
        driver.get_or_load_design(design_path, design_module, &cfg.haystack_options)?;

    let cells = design_arc.design().iter_cells().count();
    info!("Design has {} cells", cells);

    info!("Design loaded with key: {:#?}", design_key);

    let context = RecAnd::context(&driver, &cfg.haystack_options)?;
    let context = context.with_design(design_key.clone(), design_arc);

    let time_start = std::time::Instant::now();
    debug!("Starting query at {:?}", time_start);
    let rec_and_results = RecAnd::query(
        &design_key,
        &context,
        Instance::root("rec_and".to_string()),
        &cfg,
    );

    for rec_and in &rec_and_results {
        info!("Found RecAnd match {:?}", rec_and);
    }

    let count = rec_and_results.len();

    println!("{}", count);

    let time_end = std::time::Instant::now();
    debug!("Ending query at {:?}", time_end);

    let duration = time_end.duration_since(time_start);
    info!("Query completed in {:?}", duration);
    Ok(())
}

// fn main() {}
