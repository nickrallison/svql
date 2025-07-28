use crate::module::lookup;
use crate::module::traits::{RtlModuleResultTrait, RtlModuleTrait};
use crate::ports::{InPort, OutPort};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use svql_common::matches::IdString;
use svql_query_proc_macro::module;

#[module(
    file = "examples/patterns/basic/and/verilog/and.v",
    module = "and_gate",
    yosys = "./yosys/yosys",
    svql_pat_plugin_path = "./build/svql_pat_lib/libsvql_pat_lib.so"
)]
pub struct And;
