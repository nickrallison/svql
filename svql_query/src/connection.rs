use crate::ports::{InPort, OutPort};

#[derive(Clone, Debug)]

pub struct Connection {
    in_port: InPort,
    out_port: OutPort,
}

impl Connection {
    pub fn new(in_port: InPort, out_port: OutPort) -> Connection {
        Connection { in_port, out_port }
    }
}

#[macro_export]
macro_rules! connect {
    ( $module:expr, $wire1:expr , $wire2:expr ) => {
        {
            $module.connections.push($wire1.connect_to($wire2));
        }
    };
}