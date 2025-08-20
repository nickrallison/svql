use svql_driver::prelude::Driver;
use svql_query::Search;
use svql_query::instance::Instance;
use svql_query::queries::composites::dff_then_and::DffThenAnd;
use svql_query::queries::netlist::basic::and::and_gate::AndGate;
use svql_query::queries::netlist::basic::dff::Sdffe;
use svql_subgraph::config::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let driver = Driver::new()?;

    let config = Config::builder().exact_length().none().build();
    let root = Instance::root("dff_then_and".to_string());

    // Run the composite query via the trait method
    let haystack_path = Path::from("examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v");
    let haystack_module_name = "and_q_double_sdffe";

    let hits = <DffThenAnd<Search> as svql_query::composite::SearchableComposite>::query(
        &driver,
        haystack_module_name,
        &haystack_path,
        root,
        &config,
    );

    log::trace!("DffThenAnd matches={}", hits.len());

    // Sanity: sdffe.q drives either andg.a or andg.b
    for (k, h) in hits.iter().enumerate() {
        let q = h
            .sdffe
            .q
            .val
            .as_ref()
            .expect("missing sdffe.q")
            .design_cell_ref
            .expect("sdffe.q design driver");

        let a_src = h
            .andg
            .a
            .val
            .as_ref()
            .expect("missing andg.a")
            .design_cell_ref
            .expect("andg.a design source");
        let b_src = h
            .andg
            .b
            .val
            .as_ref()
            .expect("missing andg.b")
            .design_cell_ref
            .expect("andg.b design source");

        assert!(
            q == a_src || q == b_src,
            "expected sdffe.q to drive either andg.a or andg.b"
        );

        println!("hit[{k}]: {:#?}", h);
    }

    Ok(())
}
