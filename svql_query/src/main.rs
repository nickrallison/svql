use crate::ports::{Connection, InPort, OutPort};

mod query;
mod ports;
mod pat;
mod graph;

struct CAdd {
    in1: InPort,
    in2: InPort,
    out: OutPort,
    c: OutPort
}

struct Combined {
    
    connections: Vec<Connection>,
    cadd1: CAdd,
    cadd2: CAdd,
}

impl Combined {
    fn connect(mut self) -> Self {
        self = connect!(self, &self.cadd1.in1, &self.cadd2.out);
        connect!(self, &self.cadd2.c, &self.cadd1.in2)
    }
}



fn main() {
    println!("Hello, world!");
}
