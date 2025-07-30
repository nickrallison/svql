use crate::instance::Instance;
use crate::module::inst_path;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct InPort {
    pub path: Instance
}

impl InPort {
    pub fn new<S: Into<String>>(s: S, parent_path: Instance) -> InPort {
        let inst = Arc::new(s.into());
        let path: Instance = parent_path.create_child(inst);
        InPort { path }
    }

    #[allow(dead_code)]
    pub fn connect_to(&self, out_port: &OutPort) -> Connection<InPort, OutPort> {
        Connection::new(self.clone(), out_port.clone())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct OutPort {
    pub path: Instance
}

impl OutPort {
    pub fn new<S: Into<String>>(s: S, parent_path: Instance) -> OutPort {
        let inst = Arc::new(s.into());
        let path = parent_path.create_child(inst);
        OutPort { path }
    }
    #[allow(dead_code)]
    pub fn connect_to(&self, in_port: &InPort) -> Connection<InPort, OutPort> {
        Connection::new(in_port.clone(), self.clone())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Connection<In, Out> {
    pub in_port: In,
    pub out_port: Out,
}

impl<In, Out> Connection<In, Out> {
    pub fn new(in_port: In, out_port: Out) -> Connection<In, Out> {
        Connection { in_port, out_port }
    }
}

#[macro_export]
macro_rules! connect {
    ( $connections:expr, $wire1:expr , $wire2:expr ) => {{
        $connections.insert($wire1.connect_to($wire2));
    }};
}
