use crate::ports::{InPort, OutPort};

pub trait Query {
    // fn get_in_ports() -> Vec<InPort>;
    // fn get_out_ports() -> Vec<OutPort>;
    fn find_matches();
}