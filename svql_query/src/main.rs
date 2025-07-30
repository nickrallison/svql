#![allow(unused_imports)]
use crate::driver::mock::MockDriver;
use crate::driver::Driver;
use crate::examples::and::And;
use crate::examples::double_and::DoubleAnd;
use crate::examples::triple_and::TripleAnd;
// use crate::examples::double_and::DoubleAnd;
use crate::module::RtlModule;
use crate::query::traits::RtlQueryTrait;
use crate::query::RtlQuery;
use log::LevelFilter;
use svql_common::matches::SanitizedQueryMatch;

mod driver;
mod examples;
mod module;
mod ports;
// mod query;
mod instance;

fn main() {
    // Initialize logger
    env_logger::Builder::new()
        .filter_level(LevelFilter::Trace)
        .init();

    log::trace!("Starting svql_query");

    // Directly query the driver for AND gate matches
    // let triple_and: RtlQuery<TripleAnd> = RtlQuery::new(TripleAnd::new(), "triple_and".to_string());
    let and = RtlModule::<And>::root("and_gate".to_string());
    
    // let driver = Driver::new_net("localhost:9999".to_string());
    let driver = Driver::new_mock();

    match and.query(&driver) {
        Ok(matches) => {
            log::trace!("And query returned {} matches", matches.len());
            for m in matches.iter() {
                println!("-----\n{:#?}", m);
            }
        }
        Err(e) => {
            log::trace!("Triple And query failed with error: {:?}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_and_query() {
        let and = RtlModule::<And>::new(And::new(), "and_gate".to_string());
        let driver = Driver::new_mock();
        let matches = and.query(&driver);
        assert!(matches.is_ok());
        let matches = matches.unwrap();
        for m in matches.iter() {
            println!("-----\n{:#?}", m);
        }
    }
}