use std::path::Path;
use crate::connection::Connection;
use crate::file_info::Match;
use crate::ports::{InPort, OutPort};
use crate::query::{Module, Query};

mod query;
mod ports;
mod connection;
mod file_info;

struct CAdd {
    in1: InPort,
    in2: InPort,
    out: OutPort,
    c: OutPort
}

impl Module for CAdd {
    fn file_path(&self) -> &std::path::PathBuf {
        unimplemented!()
    }
}

struct Combined {
    
    connections: Vec<Connection>,
    modules: Vec<Box<dyn Module>>,
    cadd1: CAdd,
    cadd2: CAdd,
}

impl Combined {
    fn connect(mut self) -> Self {
        connect!(self, &self.cadd1.in1, &self.cadd2.out);
        connect!(self, &self.cadd2.c, &self.cadd1.in2);
        self
    }
}

impl Query for Combined {
    fn query(&self, rtlil: &Path, top: String) -> Vec<Match> {
        // Implement the query logic here
        vec![]
    }
}





fn main() {
    println!("Hello, world!");
}
