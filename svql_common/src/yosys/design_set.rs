use std::collections::BTreeMap;

use prjunnamed_netlist::Design;

// pub struct DesignSet {
//     pub top_module: String,
//     pub modules: BTreeMap<String, Design>,
// }

// impl DesignSet {
//     pub fn new(
//         top_module: String,
//         modules: BTreeMap<String, Design>,
//     ) -> Result<Self, Box<dyn std::error::Error>> {
//         if !modules.contains_key(&top_module) {
//             return Err(format!("Top module '{}' not found in modules", top_module).into());
//         }
//         Ok(Self {
//             top_module,
//             modules,
//         })
//     }
// }
