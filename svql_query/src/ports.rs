
use crate::connection::Connection;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct InPort(pub String);

impl InPort {
    pub fn new<S: Into<String>>(s: S) -> InPort { InPort(s.into()) }
    pub fn connect_to(&self, out_port: &OutPort) -> Connection {
        Connection::new(self.clone(), out_port.clone())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct OutPort(pub String);

impl OutPort {
    pub fn new<S: Into<String>>(s: S) -> OutPort { OutPort(s.into()) }
    pub fn connect_to(&self, in_port: &InPort) -> Connection {
        Connection::new(in_port.clone(), self.clone())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Connection<In, Out> {
    pub in_port: In,
    pub out_port: Out,
}

impl<In, Out> Connection<In, Out> {
    pub fn new(in_port: In, out_port: Out) -> Connection {
        Connection { in_port, out_port }
    }
}

#[macro_export]
macro_rules! connect {
    ( $module:expr, $wire1:expr , $wire2:expr ) => {
        {
            $module.connections.insert($wire1.connect_to($wire2));
        }
    };
}