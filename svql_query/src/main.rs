use std::path::PathBuf;
use crate::connection::Connection;
use crate::ports::{InPort, OutPort};
use crate::query::{Module, Query};

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

fn main() {
    let and = And {
        a: InPort::new("and.a"),
        b: InPort::new("and.b"),
        y: OutPort::new("and.y"),
    };
    
    // loopback addr:9999
    let res = and.query_net("127.0.0.1:9999");
    println!("{:?}", res);
}
