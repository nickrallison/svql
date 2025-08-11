use svql_driver_handler::YosysProc;
use svql_query::composite::SearchableComposite;
use svql_driver_handler::driver::mock::and_three::MockDriverThreeAnd;
use svql_driver_handler::driver::Driver;
use svql_query::queries::basic::and::RecursiveAnd;
use svql_query::{Match, Search, WithPath};

use crate::and::AndAB;

mod and;

fn main() {
    // env logger

    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    // let yosys_proc: YosysProc = YosysProc::new_nonblocking("examples/patterns/basic/and/many_ands_2.v".into(), "many_ands".into()).unwrap();
    let yosys_proc: YosysProc = YosysProc::new("examples/patterns/basic/and/many_ands_2.v".into(), "many_ands".into()).unwrap();



    let driver = yosys_proc.driver();

    let cmd = yosys_proc.get_command();

    let _ = yosys_proc.kill();

    println!("Yosys Command: {}", cmd);

    let and_ab = AndAB::<Search>::root("rec_and");
    let and_ab_search_result: Vec<AndAB<Match>> =
        AndAB::<Search>::query(&driver, and_ab.path());
    assert_eq!(
        and_ab_search_result.len(),
        6,
        "Expected 6 matches for AndAB, got {}",
        and_ab_search_result.len()
    );

}
