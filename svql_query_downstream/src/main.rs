
use std::path::{Path, PathBuf};

use svql_query::composite::SearchableComposite;
use svql_driver_handler::Driver;
use svql_query::{Match, Search, WithPath};

use crate::and::AndAB;

mod and;

fn main() {
    // env logger

    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let design = PathBuf::from("examples/patterns/basic/and/many_ands_2.v");
    let module_name = "many_ands".to_string();

    let driver = Driver::new_proc(design, module_name).expect("Failed to create proc driver");

    let and_ab = AndAB::<Search>::root("rec_and");
    let and_ab_search_result: Vec<AndAB<Match>> =
        AndAB::<Search>::query(&driver, and_ab.path());
    assert_eq!(
        and_ab_search_result.len(),
        1,
        "Expected 1 match for AndAB, got {}",
        and_ab_search_result.len()
    );

}
