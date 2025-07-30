// #![allow(unused_imports)]
// use crate::driver::mock::MockDriver;
// use crate::driver::Driver;
// use crate::examples::and::And;
// use crate::examples::double_and::DoubleAnd;
// use crate::examples::triple_and::TripleAnd;
// use crate::module::RtlModule;
// use crate::query::traits::RtlQueryTrait;
// use crate::query::RtlQuery;
// use log::LevelFilter;
// use svql_common::matches::SanitizedQueryMatch;

// mod driver;
// mod examples;
// mod module;
// mod port_spec;
mod instance;

// fn main() {
//     // Initialize logger
//     env_logger::Builder::new()
//         .filter_level(LevelFilter::Trace)
//         .init();

//     log::trace!("Starting svql_query");

//     // Directly query the driver for AND gate matches
//     // let triple_and: RtlQuery<TripleAnd> = RtlQuery::new(TripleAnd::new(), "triple_and".to_string());
//     let and = RtlModule::<And>::root("and_gate".to_string());
    
//     // let driver = Driver::new_net("localhost:9999".to_string());
//     let driver = Driver::new_mock();

//     match and.query(&driver) {
//         Ok(matches) => {
//             log::trace!("And query returned {} matches", matches.len());
//             for m in matches.iter() {
//                 println!("-----\n{:#?}", m);
//             }
//         }
//         Err(e) => {
//             log::trace!("Triple And query failed with error: {:?}", e);
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_and_query() {
//         let and = RtlModule::<And>::new(And::new(), "and_gate".to_string());
//         let driver = Driver::new_mock();
//         let matches = and.query(&driver);
//         assert!(matches.is_ok());
//         let matches = matches.unwrap();
//         for m in matches.iter() {
//             println!("-----\n{:#?}", m);
//         }
//     }
// }

use crate::instance::Instance;
// use crate::module::lookup;
// use crate::module::traits::{RtlModuleResultTrait, RtlModuleTrait};
// use crate::ports::{InPort, OutPort};
// use std::collections::{HashMap, VecDeque};
// use std::path::PathBuf;
// use std::sync::Arc;
// use svql_common::id_string::IdString;
// use svql_query_proc_macro::module;

// #[module(
//     file = "examples/patterns/basic/and/verilog/and.v",
//     module = "and_gate",
//     yosys = "./yosys/yosys",
//     svql_pat_plugin_path = "./build/svql_pat_lib/libsvql_pat_lib.so"
// )]
// pub struct And;



// ########################
// Base Search Types
// ########################

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Search;
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Match;

// ########################
// Containers
// ########################

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wire<T> {
    pub path: Instance,
    pub val: T,
}

impl <T> Wire<T> where T: Default {
    pub fn root(name: String) -> Self {
        let path = Instance::root(name);
        Self::new(path)
    }
    pub fn new(path: Instance) -> Self {
        Self { path, val: T::default() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct And<T> {
    pub path: Instance,
    pub a: Wire<T>,
    pub b: Wire<T>,
    pub y: Wire<T>,
}

impl<T: Default> And<T> {
    pub fn root(name: String) -> Self {
        let path = Instance::root(name);
        Self::new(path)
    }
    pub fn new(path: Instance) -> Self {
        let a = Wire::new(path.child("a".to_string()));
        let b = Wire::new(path.child("b".to_string()));
        let y = Wire::new(path.child("y".to_string()));
        Self { path, a, b, y }
    }
}

impl And<Search> {
    pub fn query(&self) -> Vec<And<Match>> {
       todo!();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoubleAnd<T> {
    pub path: Instance,
    pub and1: And<T>,
    pub and2: And<T>,
}

impl<T: Default> DoubleAnd<T> {
    pub fn root(name: String) -> Self {
        let path = Instance::root(name);
        Self::new(path)
    }
    pub fn new(path: Instance) -> Self {
        let and1 = And::new(path.child("and1".to_string()));
        let and2 = And::new(path.child("and2".to_string()));
        Self { path, and1, and2 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TripleAnd<T> {
    pub path: Instance,
    pub double_and: DoubleAnd<T>,
    pub and: And<T>,
}

impl<T: Default> TripleAnd<T> {
    pub fn root(name: String) -> Self {
        let path = Instance::root(name);
        Self::new(path)
    }
    pub fn new(path: Instance) -> Self {
        let double_and = DoubleAnd::new(path.child("double_and".to_string()));
        let and = And::new(path.child("and".to_string()));
        Self { path, double_and, and }
    }
}

fn main() {
    let and_search: And<Search> = And::root("and".to_string());
    let double_and_search: DoubleAnd<Search> = DoubleAnd::root("double_and".to_string());
    let triple_and_search: TripleAnd<Search> = TripleAnd::root("triple_and".to_string());

    // let and_matches: Vec<And<Match>> = and_search.query().unwrap();
    // let double_and_matches: Vec<DoubleAnd<Match>> = double_and_search.query().unwrap();
    // let triple_and_matches: Vec<TripleAnd<Match>> = triple_and_search.query().unwrap();
    //... 
}