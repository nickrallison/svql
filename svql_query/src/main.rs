#![allow(unused_imports)]
use crate::driver::mock::MockDriver;
use crate::driver::Driver;
use crate::examples::and::And;
use crate::examples::double_and::DoubleAnd;
use crate::module::RtlModule;
use crate::query::traits::RtlQueryTrait;
use crate::query::RtlQuery;
use log::LevelFilter;
use svql_common::mat::SanitizedQueryMatch;

mod driver;
mod examples;
mod module;
mod ports;
mod query;

fn print_match_details(m: &SanitizedQueryMatch, name: &str) {
    log::trace!("{} match details:", name);
    log::trace!("  Port map:");
    for (k, v) in &m.port_map {
        log::trace!("    {:?} => {:?}", k, v);
    }
    log::trace!("  Cell map:");
    for (k, v) in &m.cell_map {
        log::trace!("    {:?} => {:?}", k, v);
    }
}

fn main() {
    // Initialize logger
    env_logger::Builder::new()
        .filter_level(LevelFilter::Trace)
        .init();

    log::trace!("Starting svql_query");

    // Use mock driver for testing
    let double_and: RtlQuery<DoubleAnd> = RtlQuery::new(DoubleAnd::new(), "double_and".to_string());

    let driver = Driver::Mock(MockDriver);

    let matches = double_and.query(&driver).unwrap();
    assert_eq!(matches.len(), 2, "Expected 2 matches for DoubleAnd query");

    // Directly query the driver for AND gate matches
    // let and: RtlModule<And> = RtlModule::new(And::new(), "and_gate".to_string());
    // let cfg = and.config();
    //
    // match driver.query(&cfg) {
    //     Ok(matches) => {
    //         log::trace!("Direct AND query returned {} matches", matches.len());
    //         for (i, m) in matches.iter().enumerate() {
    //             print_match_details(m, &format!("Direct AND match {}", i));
    //             println!("serialized match: {:#?}", m);
    //         }
    //     }
    //     Err(e) => {
    //         log::trace!("Direct AND query failed with error: {:?}", e);
    //     }
    // }
}
