use std::path::PathBuf;
use std::net::ToSocketAddrs;
use crate::connection::Connection;
use crate::ports::{InPort, OutPort};
use crate::query::{Module, Query};
use crate::net::{SvqlQueryError};
use svql_common::mat::{SanitizedQueryMatch, IdStringError};

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
    fn query<P: Into<PathBuf>, S: Into<String>>(&self, design_path: P, top: S) -> Vec<SanitizedQueryMatch> {
        todo!()
    }

    fn query_net<A: ToSocketAddrs>(&self, addr: A) -> Result<Vec<SanitizedQueryMatch>, SvqlQueryError> {
        let and_results = self.and1.query_net(&addr)?;
        let and2_results = self.and2.query_net(&addr)?;

        todo!()
        
        // trim all results to only include matches where the connection is valid
        // let mut results = Vec::new();
        // for match1 in and_results {
        //     // filter for results in match2 that match the connection criteria
        //     // i.e. the output of and1 is connected to the input of and2
        //     let match1_y_id_str = match1.port_map.get("\\a");
        //     let match1_y = match1.port_map.get("\\y"); 

        //     for match2 in &and2_results {
        //         // connections only specify connectivity, not the actual names

        //         if match1.port_map.contains_key(&self.and1.y.idstring) && 
        //            match2.port_map.contains_key(&self.and2.y.idstring) {
        //             let mut combined_match = match1.clone();
        //             combined_match.port_map.extend(match2.port_map.clone());
        //             results.push(combined_match);
        //         }
        //     }
        // }
        // Ok(results)
    }

}

fn main() {
    let and = And {
        a: InPort::new("and.a"),
        b: InPort::new("and.b"),
        y: OutPort::new("and.y"),
    };

    let res = and.query_net("127.0.0.1:9999");
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
        and1: and,
        and2: and2,
    }.connect();
    
    // loopback addr:9999
    let res = combined.query_net("127.0.0.1:9999");
    if res.is_err() {
        eprintln!("Error querying net: {:?}", res.err());
        return;
    }
    let res = res.unwrap();
    let pretty = serde_json::to_string_pretty(&res).unwrap();
    println!("{}", pretty);
}
