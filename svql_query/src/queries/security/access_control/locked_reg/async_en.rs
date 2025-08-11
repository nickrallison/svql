use crate::instance::Instance;
use crate::netlist::{Netlist, SearchableNetlist};
use crate::{lookup, Match, QueryMatch, Search, State, Wire, WithPath};

// input [15:0] data_in,
// input clk,
// input resetn,
// input write_en,
// output reg [15:0] data_out

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncEnLockedReg<S>
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

impl<S> WithPath<S> for AsyncEnLockedReg<S>
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

    crate::impl_find_port!(AsyncEnLockedReg, data_in, clk, resetn, write_en, data_out);

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Netlist<S> for AsyncEnLockedReg<S>
where
    S: State,
{
    const MODULE_NAME: &'static str = "async_en";
    const FILE_PATH: &'static str = "examples/patterns/security/access_control/locked_reg/rtlil/async_en.il";
    const YOSYS: &'static str = "./yosys/yosys";
    const SVQL_DRIVER_PLUGIN: &'static str = "./build/svql_driver/libsvql_driver.so";
}

impl SearchableNetlist for AsyncEnLockedReg<Search> {
    type Hit = AsyncEnLockedReg<Match>;

    fn from_query_match(m: QueryMatch, path: Instance) -> Self::Hit {
        let data_in = Match {
            id: lookup(&m.port_map, "data_in").cloned().unwrap(),
        };
        let clk = Match {
            id: lookup(&m.port_map, "clk").cloned().unwrap(),
        };
        let resetn = Match {
            id: lookup(&m.port_map, "resetn").cloned().unwrap(),
        };
        let write_en = Match {
            id: lookup(&m.port_map, "write_en").cloned().unwrap(),
        };
        let data_out = Match {
            id: lookup(&m.port_map, "data_out").cloned().unwrap(),
        };
        AsyncEnLockedReg::<Match> {
            path: path.clone(),
            data_in: Wire::with_val(path.child("data_in".into()), data_in),
            clk: Wire::with_val(path.child("clk".into()), clk),
            resetn: Wire::with_val(path.child("resetn".into()), resetn),
            write_en: Wire::with_val(path.child("write_en".into()), write_en),
            data_out: Wire::with_val(path.child("data_out".into()), data_out),
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
    fn test_async_en_locked_reg_netlist() {

        let design = PathBuf::from("examples/patterns/security/access_control/locked_reg/rtlil/aasync_en.il");
        let module_name = "async_en".to_string();

        let driver = Driver::new_proc(design, module_name).expect("Failed to create proc driver");


        let async_en = AsyncEnLockedReg::<Search>::root("async_en".to_string());
        assert_eq!(async_en.path().inst_path(), "async_en");
        assert_eq!(async_en.data_in.path.inst_path(), "async_en.data_in");
        assert_eq!(async_en.clk.path.inst_path(), "async_en.clk");
        assert_eq!(async_en.resetn.path.inst_path(), "async_en.resetn");
        assert_eq!(async_en.write_en.path.inst_path(), "async_en.write_en");
        assert_eq!(async_en.data_out.path.inst_path(), "async_en.data_out");

        let async_en_search_result = AsyncEnLockedReg::<Search>::query(&driver, async_en.path());
        assert_eq!(
            async_en_search_result.len(),
            1,
            "Expected 1 match for AsyncEnLockedReg, got {}",
            async_en_search_result.len()
        );
    }
}
