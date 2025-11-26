use crate::{State, Wire};
use svql_macros::netlist;

#[netlist(file = "examples/patterns/basic/ff/verilog/sdffe.v", name = "sdffe")]
pub struct Sdffe<S: State> {
    pub clk: Wire<S>,
    pub d: Wire<S>,
    pub reset: Wire<S>,
    pub en: Wire<S>,
    pub q: Wire<S>,
}

#[netlist(file = "examples/patterns/basic/ff/rtlil/adffe.il", name = "adffe")]
pub struct Adffe<S: State> {
    pub clk: Wire<S>,
    pub d: Wire<S>,
    pub reset_n: Wire<S>,
    pub en: Wire<S>,
    pub q: Wire<S>,
}

#[netlist(file = "examples/patterns/basic/ff/rtlil/sdff.il", name = "sdff")]
pub struct Sdff<S: State> {
    pub clk: Wire<S>,
    pub d: Wire<S>,
    pub reset: Wire<S>,
    pub q: Wire<S>,
}

#[netlist(file = "examples/patterns/basic/ff/rtlil/adff.il", name = "adff")]
pub struct Adff<S: State> {
    pub clk: Wire<S>,
    pub d: Wire<S>,
    pub reset_n: Wire<S>,
    pub q: Wire<S>,
}

#[netlist(file = "examples/patterns/basic/ff/rtlil/dffe.il", name = "dffe")]
pub struct Dffe<S: State> {
    pub clk: Wire<S>,
    pub d: Wire<S>,
    pub en: Wire<S>,
    pub q: Wire<S>,
}
