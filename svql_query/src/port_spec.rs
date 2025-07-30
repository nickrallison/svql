use crate::instance::Instance;
use crate::module::inst_path;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct QueryPort {
    pub path: Instance
}
impl QueryPort {
    pub fn new<S: Into<String>>(s: S, parent_path: Instance) -> QueryPort {
        let inst = Arc::new(s.into());
        let path: Instance = parent_path.create_child(inst);
        QueryPort { path }
    }

    pub fn connect_to(&self, out_port: QueryPort) -> Connection<QueryPort> {
        Connection::new(self.clone(), out_port.clone())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Connection<Port> {
    pub in_port: Port,
    pub out_port: Port,
}

impl<Port> Connection<Port> {
    pub fn new(in_port: Port, out_port: Port) -> Connection<Port> {
        Connection { in_port, out_port }
    }
}
