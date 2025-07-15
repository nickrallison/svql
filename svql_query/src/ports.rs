use std::ffi::OsStr;
use std::path::PathBuf;
use std::str::FromStr;
use crate::connection::Connection;
use crate::file_info::Match;

#[derive(Clone, Debug)]
pub struct InPort(String);

impl InPort {
    pub fn new<S: Into<String>>(s: S) -> InPort {
        InPort(s.into())
    }
    pub fn connect_to(&self, out_port: &OutPort) -> Connection {
        Connection::new(self.clone(), out_port.clone())
    }
}

#[derive(Clone, Debug)]
pub struct OutPort(String);

impl OutPort {
    pub fn new<S: Into<String>>(s: S) -> OutPort {
        OutPort(s.into())
    }

    pub fn connect_to(&self, in_port: &InPort) -> Connection {
        Connection::new(in_port.clone(), self.clone())
    }
}
