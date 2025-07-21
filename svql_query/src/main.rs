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
        connect!(self, &self.and1.a, &self.and2.y);
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
        todo!();
    }

}

fn main() {
    let and = And {
        a: InPort::new("and.a"),
        b: InPort::new("and.b"),
        y: OutPort::new("and.y"),
    };
    
    // loopback addr:9999
    let res = and.query_net("127.0.0.1:9999");
    if res.is_err() {
        eprintln!("Error querying net: {:?}", res.err());
        return;
    }
    let res = res.unwrap();
    let pretty = serde_json::to_string_pretty(&res).unwrap();
    println!("{}", pretty);
}
