use crate::module::inst_path;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct InPort {
    pub height: usize,
    pub inst: Arc<String>,
    pub full_path: VecDeque<Arc<String>>,
}

impl InPort {
    pub fn new<S: Into<String>>(s: S) -> InPort {
        let inst = Arc::new(s.into());
        let mut in_port = InPort {
            inst,
            full_path: vec![].into(),
            height: 0,
        };
        in_port.init_full_path(vec![].into(), 0);
        in_port
    }
    #[allow(dead_code)]
    pub fn connect_to(&self, out_port: &OutPort) -> Connection<InPort, OutPort> {
        Connection::new(self.clone(), out_port.clone())
    }

    pub fn init_full_path(&mut self, parent_path: VecDeque<Arc<String>>, height: usize) {
        let mut full_path = parent_path.clone();
        full_path.push_back(self.inst.clone());
        self.full_path = full_path;
        self.height = height;
    }
    #[allow(dead_code)]
    pub fn inst_path(&self) -> String {
        inst_path(&self.full_path)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct OutPort {
    pub height: usize,
    pub inst: Arc<String>,
    pub full_path: VecDeque<Arc<String>>,
}

impl OutPort {
    pub fn new<S: Into<String>>(s: S) -> OutPort {
        let inst = Arc::new(s.into());
        let mut out_port = OutPort {
            inst,
            full_path: vec![].into(),
            height: 0,
        };
        out_port.init_full_path(vec![].into(), 0);
        out_port
    }
    #[allow(dead_code)]
    pub fn connect_to(&self, in_port: &InPort) -> Connection<InPort, OutPort> {
        Connection::new(in_port.clone(), self.clone())
    }

    pub fn init_full_path(&mut self, parent_path: VecDeque<Arc<String>>, height: usize) {
        let mut full_path = parent_path.clone();
        full_path.push_back(self.inst.clone());
        self.full_path = full_path;
        self.height = height;
    }

    #[allow(dead_code)]
    pub fn inst_path(&self) -> String {
        inst_path(&self.full_path)
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
