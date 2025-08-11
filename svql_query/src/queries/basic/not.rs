// ########################
// Examples
// ########################

use crate::instance::Instance;
use crate::netlist::{Netlist, SearchableNetlist};
use crate::{lookup, Match, QueryMatch, Search, State, Wire, WithPath};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Not<S>
where
    S: State,
{
    pub path: Instance,
    pub a: Wire<S>,
    pub y: Wire<S>,
}

impl<S> WithPath<S> for Not<S>
where
    S: State,
{
    fn new(path: Instance) -> Self {
        let a = Wire::new(path.child("a".to_string()));
        let y = Wire::new(path.child("y".to_string()));
        Self { path, a, y }
    }

    crate::impl_find_port!(Not, a, y);

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Netlist<S> for Not<S>
where
    S: State,
{
    const MODULE_NAME: &'static str = "not_gate";
    const FILE_PATH: &'static str = "./examples/patterns/basic/not/not.v";
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
