use std::path::PathBuf;
use std::net::ToSocketAddrs;
use crate::connection::Connection;
use crate::ports::{InPort, OutPort};
use crate::query::{Module, Query};
use crate::net::{SvqlQueryError};
use svql_common::mat::{SanitizedQueryMatch, IdStringError, IdString};

mod query;
mod ports;
mod connection;
mod net;

// struct AndGate {
//     in1: InPort,
//     in2: InPort,
//     out: OutPort,
// }

// impl Module for AndGate {
//     fn file_path(&self) -> PathBuf {
//         "svql_query/verilog/and.v".into()
//     }
//     fn module_name(&self) -> String {
//         "and_gate".to_string()
//     }
// }

// struct Combined {
    
//     connections: Vec<Connection>,
//     // modules: Vec<Box<dyn Module>>,
//     cadd1: CAdd,
//     cadd2: CAdd,
// }

// impl Combined {
//     fn connect(mut self) -> Self {
//         connect!(self, &self.cadd1.in1, &self.cadd2.out);
//         connect!(self, &self.cadd2.c, &self.cadd1.in2);
//         self
//     }
// }

// impl Query for Combined {
//     fn query<P: Into<PathBuf>, S: Into<String>>(&self, design_path: P, top: S) -> Vec<Match> {
//         // Implement the query logic here
//         vec![]
//     }
// }

struct And {
    a: InPort,
    b: InPort,
    y: OutPort,
}

impl Module for And {
    fn file_path(&self) -> PathBuf {
        PathBuf::from("svql_query/verilog/and.v")
    }
    fn module_name(&self) -> String {
        "and_gate".to_string()
    }
}

struct CombinedAnd {
    connections: Vec<Connection>,
    and1: And,
    and2: And,
}

impl CombinedAnd {
    fn connect(mut self) -> Self {
        connect!(self, &self.and2.a, &self.and1.y);
        self
    }
}

impl Query for CombinedAnd {
    fn query<P: Into<PathBuf>, S: Into<String>>(
        &self,
        design_path: P,
        top: S,
    ) -> Vec<SanitizedQueryMatch> {
        todo!()
    }

    fn query_net<A: ToSocketAddrs>(
        &self,
        addr: A,
    ) -> Result<Vec<SanitizedQueryMatch>, SvqlQueryError> {
        // 1. run both sub-queries
        let and1_results = self.and1.query_net(&addr)?;
        let and2_results = self.and2.query_net(&addr)?;

        // 2. port names we care about (hard-coded for the two-AND example)
        let key_and1_y = IdString::Named("y".to_owned());
        let key_and2_a = IdString::Named("a".to_owned());

        let mut merged = Vec::<SanitizedQueryMatch>::new();

        // 3. brute-force pairwise merge whenever the connectivity matches
        for m1 in &and1_results {
            let Some(hay_of_y) = m1.port_map.get(&key_and1_y) else { continue };

            for m2 in &and2_results {
                let Some(hay_of_a) = m2.port_map.get(&key_and2_a) else { continue };

                // and1.y ───► same signal ◄─── and2.a   ?
                if hay_of_y == hay_of_a {
                    // build the union; keep first-seen value on duplicates
                    let mut port_map: HashMap<IdString, IdString> = m1.port_map.clone();
                    for (k, v) in &m2.port_map {
                        port_map.entry(k.clone()).or_insert(v.clone());
                    }

                    let mut cell_map = m1.cell_map.clone();
                    for (k, v) in &m2.cell_map {
                        cell_map.entry(k.clone()).or_insert(v.clone());
                    }

                    merged.push(SanitizedQueryMatch { port_map, cell_map });
                }
            }
        }

        Ok(merged)
    }
}

use std::collections::hash_map::Entry;
use std::collections::HashMap;
// Helper ────────────────────────────────────────────────────────────────────────
fn try_merge_matches(
    m1: &SanitizedQueryMatch,
    m2: &SanitizedQueryMatch,
) -> Option<SanitizedQueryMatch> {
    // merge port maps – abort if the same key maps to different hay-stack ids
    let mut port_map: HashMap<IdString, IdString> = m1.port_map.clone();
    for (k, v) in &m2.port_map {
        match port_map.entry(k.clone()) {
            Entry::Vacant(e) => {
                e.insert(v.clone());
            }
            Entry::Occupied(e) => {
                if e.get() != v {
                    return None; // conflict
                }
            }
        }
    }

    // merge cell maps – same policy as above
    let mut cell_map = m1.cell_map.clone();
    for (k, v) in &m2.cell_map {
        match cell_map.entry(k.clone()) {
            Entry::Vacant(e) => {
                e.insert(v.clone());
            }
            Entry::Occupied(e) => {
                if e.get() != v {
                    return None; // conflict
                }
            }
        }
    }

    Some(SanitizedQueryMatch { port_map, cell_map })
}

fn main() {
    let and1 = And {
        a: InPort::new("and1.a"),
        b: InPort::new("and1.b"),
        y: OutPort::new("and1.y"),
    };

    let res = and1.query_net("127.0.0.1:9999");
    if res.is_err() {
        eprintln!("Error querying net: {:?}", res.err());
        return;
    }
    let res = res.unwrap();
    for match_item in &res {
        println!("{}", match_item);
    }
    // let pretty = serde_json::to_string_pretty(&res).unwrap();
    // println!("{}", pretty);


    let and2 = And {
        a: InPort::new("and2.a"),
        b: InPort::new("and2.b"),
        y: OutPort::new("and2.y"),
    };
    let combined = CombinedAnd {
        connections: Vec::new(),
        and1: and1,
        and2: and2,
    }.connect();
    
    let res = combined.query_net("127.0.0.1:9999");
    if res.is_err() {
        eprintln!("Error querying net: {:?}", res.err());
        return;
    }
    let res = res.unwrap();
    for match_item in &res {
        println!("{}", match_item);
    }
}
