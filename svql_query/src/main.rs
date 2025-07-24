use crate::driver::Driver;
use crate::examples::and::And;
use crate::module::RtlModule;
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
    let driver = Driver::new_mock();

    // Directly query the driver for AND gate matches
    let and: RtlModule<And> = RtlModule::new("and_gate".to_string(), And::new());
    let cfg = and.config();

    match driver.query(&cfg) {
        Ok(matches) => {
            log::trace!("Direct AND query returned {} matches", matches.len());
            for (i, m) in matches.iter().enumerate() {
                print_match_details(m, &format!("Direct AND match {}", i));
                println!("serialized match: {:#?}", m);
            }
        }
        Err(e) => {
            log::trace!("Direct AND query failed with error: {:?}", e);
        }
    }
}
