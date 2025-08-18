// ########################
// Examples
// ########################

use std::sync::{Arc, RwLock};

use svql_driver::SubgraphMatch;

use crate::instance::Instance;
use crate::netlist::{Netlist, SearchableNetlist};
use crate::{Match, Search, State, Wire, WithPath};
use crate::lookup;

#[derive(Debug, Clone)]
pub struct And<S>
where
    S: State,
{
    pub driver: svql_driver::Driver,
    pub path: Instance,
    pub a: Wire<S>,
    pub b: Wire<S>,
    pub y: Wire<S>,
}


impl<S> WithPath<S> for And<S>
where
    S: State,
{
    crate::impl_find_port!(And, a, b, y);

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Netlist<S> for And<S>
where
    S: State,
{
    const MODULE_NAME: &'static str = "and_gate";
    const FILE_PATH: &'static str = "./examples/patterns/basic/and/and.v";

    fn driver(&self) -> &svql_driver::Driver {
        &self.driver
    }
}

impl SearchableNetlist for And<Search> {
    type Hit<'p, 'd> = And<Match<'p, 'd>>;

    fn from_query_match<'p, 'd>(m: &SubgraphMatch<'p, 'd>, path: Instance) -> Self::Hit<'p, 'd> {
        // let pat_a_cell_ref = m.pat_input_cells
        //     .get("a")
        //     .expect("Expected 'a' input cell in match");
        // let pat_b_cell_ref = m.pat_input_cells
        //     .get("b")
        //     .expect("Expected 'b' input cell in match");
        // let pat_y_cell_ref = m.pat_output_cells
        //     .get("y")
        //     .expect("Expected 'y' output cell in match");

        // let design_a_cell_ref = m.cell_mapping
        //     .get(pat_a_cell_ref)
        //     .expect("Expected 'a' input cell mapping in match");

        // let design_b_cell_ref = m.cell_mapping
        //     .get(pat_b_cell_ref)
        // .expect("Expected 'b' input cell mapping in match");        

        // let design_y_cell_ref = m.cell_mapping
        //     .get(pat_y_cell_ref)
        //     .expect("Expected 'y' output cell mapping in match");   
       
        todo!()
    }
}

// #[cfg(test)]
// mod tests {

//     use std::path::PathBuf;
//     use svql_driver::Driver;

//     use super::*;

//     // ###############
//     // Netlist Tests
//     // ###############
//     #[test]
//     fn test_and_netlist() {
//         let design = PathBuf::from("examples/patterns/basic/and/and_tree.v");
//         let module_name = "and_tree".to_string();

//         let driver = Driver::new(design, module_name);

//         let and = And::<Search>::root("and".to_string());
//         assert_eq!(and.path().inst_path(), "and");
//         assert_eq!(and.a.path.inst_path(), "and.a");
//         assert_eq!(and.b.path.inst_path(), "and.b");
//         assert_eq!(and.y.path.inst_path(), "and.y");

//         let and_search_result = And::<Search>::query(&driver, and.path());
//         assert_eq!(
//             and_search_result.len(),
//             4,
//             "Expected 4 matches for And, got {}",
//             and_search_result.len()
//         );
//     }
// }
