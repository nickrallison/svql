#[derive(Debug)]
pub struct Config {
    needle: prjunnamed_netlist::Design,
}

// #[derive(Debug, Clone)]
// pub struct Config {
//     pub pat_module_name: String,
//     pub pat_filename: String,

//     pub verbose: bool,
//     pub const_ports: bool,
//     pub nodefaultswaps: bool,
//     pub compat_pairs: Vec<CompatPair>,
//     pub swap_ports: Vec<SwapPort>,
//     pub perm_ports: Vec<PermPort>,
//     pub cell_attr: Vec<String>,
//     pub wire_attr: Vec<String>,
//     pub ignore_params: bool,
//     pub ignored_parameters: Vec<IgnoreParam>,
//     pub max_fanout: i32,
// }

// #[derive(Debug, Default, Clone)]
// pub struct CompatPair {
//     pub needle: String,
//     pub haystack: String,
// }

// #[derive(Debug, Default, Clone)]
// pub struct SwapPort {
//     pub type_name: String,
//     pub ports: Vec<String>,
// }

// #[derive(Debug, Default, Clone)]
// pub struct PermPort {
//     pub type_name: String,
//     pub left: Vec<String>,
//     pub right: Vec<String>,
// }

// #[derive(Debug, Default, Clone)]
// pub struct IgnoreParam {
//     pub param_name: String,
//     pub param_value: String,
// }

// impl Default for Config {
//     fn default() -> Self {
//         Config {
//             pat_module_name: String::new(),
//             pat_filename: String::new(),
//             verbose: false,
//             const_ports: false,
//             nodefaultswaps: false,
//             compat_pairs: Vec::new(),
//             swap_ports: Vec::new(),
//             perm_ports: Vec::new(),
//             cell_attr: Vec::new(),
//             wire_attr: Vec::new(),
//             ignore_params: false,
//             ignored_parameters: Vec::new(),
//             max_fanout: -1,
//         }
//     }
// }

// impl CompatPair {
//     pub fn new(needle: String, haystack: String) -> Self {
//         CompatPair { needle, haystack }
//     }
// }

// impl SwapPort {
//     pub fn new(type_name: String, ports: Vec<String>) -> Self {
//         SwapPort { type_name, ports }
//     }
// }

// impl PermPort {
//     pub fn new(type_name: String, left: Vec<String>, right: Vec<String>) -> Self {
//         PermPort {
//             type_name,
//             left,
//             right,
//         }
//     }
// }

// impl IgnoreParam {
//     pub fn new(param_name: String, param_value: String) -> Self {
//         IgnoreParam {
//             param_name,
//             param_value,
//         }
//     }
// }
