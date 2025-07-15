
#[derive(Clone, Debug)]
pub struct InPort(String);

impl InPort {
    pub fn new(s: String) -> InPort {
        InPort(s)
    }
    pub fn connect_to(&self, out_port: &OutPort) -> Connection {
        Connection::new(self.clone(), out_port.clone())
    }
}

#[derive(Clone, Debug)]
pub struct OutPort(String);

impl OutPort {
    pub fn new(s: String) -> OutPort {
        OutPort(s)
    }

    pub fn connect_to(&self, in_port: &InPort) -> Connection {
        Connection::new(in_port.clone(), self.clone())
    }
}

#[derive(Clone, Debug)]
pub struct Connection {
    in_port: InPort,
    out_port: OutPort,
}

impl Connection {
    fn new(in_port: InPort, out_port: OutPort) -> Connection {
        Connection { in_port, out_port }
    }
}

// export
#[macro_export]
macro_rules! connect {
    ( $module:expr, $wire1:expr , $wire2:expr ) => {
        {
            $module.connections.push($wire1.connect_to($wire2));
            $module
        }
    };
}