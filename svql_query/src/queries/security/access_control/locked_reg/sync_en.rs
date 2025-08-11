use crate::instance::Instance;
use crate::netlist::{Netlist, SearchableNetlist};
use crate::{lookup, Match, QueryMatch, Search, State, Wire, WithPath};

// input [15:0] data_in,
// input clk,
// input resetn,
// input write_en,
// output reg [15:0] data_out

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncEnLockedReg<S>
where
    S: State,
{
    pub path: Instance,
    pub data_in: Wire<S>,
    pub clk: Wire<S>,
    pub resetn: Wire<S>,
    pub write_en: Wire<S>,
    pub data_out: Wire<S>
}

impl<S> WithPath<S> for SyncEnLockedReg<S>
where
    S: State,
{
    fn new(path: Instance) -> Self {
        let data_in = Wire::new(path.child("data_in".to_string()));
        let clk = Wire::new(path.child("clk".to_string()));
        let resetn = Wire::new(path.child("resetn".to_string()));
        let write_en = Wire::new(path.child("write_en".to_string()));
        let data_out = Wire::new(path.child("data_out".to_string()));
        Self { path, data_in, clk, resetn, write_en, data_out }
    }

    crate::impl_find_port!(SyncEnLockedReg, data_in, clk, resetn, write_en, data_out);

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Netlist<S> for SyncEnLockedReg<S>
where
    S: State,
{
    const MODULE_NAME: &'static str = "not_gate";
    const FILE_PATH: &'static str = "./examples/patterns/access_control/locked_register/rtlil/dffe.il";
    const YOSYS: &'static str = "./yosys/yosys";
    const SVQL_DRIVER_PLUGIN: &'static str = "./build/svql_driver/libsvql_driver.so";
}

impl SearchableNetlist for Not<Search> {
    type Hit = Not<Match>;

    fn from_query_match(m: QueryMatch, path: Instance) -> Self::Hit {
        let a = Match {
            id: lookup(&m.port_map, "a").cloned().unwrap(),
        };
        let y = Match {
            id: lookup(&m.port_map, "y").cloned().unwrap(),
        };
        Not::<Match> {
            path: path.clone(),
            a: Wire::with_val(path.child("a".into()), a),
            y: Wire::with_val(path.child("y".into()), y),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use svql_driver_handler::Driver;

    use crate::Search;

    // ###############
    // Netlist Tests
    // ###############
    #[test]
    fn test_not_netlist() {

        let design = PathBuf::from("examples/patterns/basic/not/many_nots.v");
        let module_name = "many_nots".to_string();

        let driver = Driver::new_proc(design, module_name).expect("Failed to create proc driver");


        let not = Not::<Search>::root("not".to_string());
        assert_eq!(not.path().inst_path(), "not");
        assert_eq!(not.a.path.inst_path(), "not.a");
        assert_eq!(not.y.path.inst_path(), "not.y");

        let not_search_result = Not::<Search>::query(&driver, not.path());
        assert_eq!(
            not_search_result.len(),
            2,
            "Expected 2 matches for Not, got {}",
            not_search_result.len()
        );
    }
}
