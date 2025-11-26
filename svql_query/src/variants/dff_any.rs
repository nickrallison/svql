use crate::primitives::dff::{Adff, Adffe, Dffe, Sdff, Sdffe};
use crate::{State, Wire};
use svql_macros::variant;

#[variant(ports(clk, d, q))]
pub enum DffAny<S: State> {
    #[variant(map(clk = "clk", d = "d", q = "q"))]
    Sdffe(Sdffe<S>),

    #[variant(map(clk = "clk", d = "d", q = "q"))]
    Adffe(Adffe<S>),

    #[variant(map(clk = "clk", d = "d", q = "q"))]
    Sdff(Sdff<S>),

    #[variant(map(clk = "clk", d = "d", q = "q"))]
    Adff(Adff<S>),

    #[variant(map(clk = "clk", d = "d", q = "q"))]
    Dffe(Dffe<S>),
}

impl<S: State> DffAny<S> {
    pub fn dff_type(&self) -> &'static str {
        match self {
            DffAny::Sdffe(_) => "Sync Reset Enable DFF",
            DffAny::Adffe(_) => "Async Reset Enable DFF",
            DffAny::Sdff(_) => "Sync Reset DFF",
            DffAny::Adff(_) => "Async Reset DFF",
            DffAny::Dffe(_) => "Enable DFF",
            _ => "Unknown",
        }
    }

    pub fn reset_wire(&self) -> Option<&Wire<S>> {
        match self {
            DffAny::Sdffe(dff) => Some(&dff.reset),
            DffAny::Adffe(dff) => Some(&dff.reset_n),
            DffAny::Sdff(dff) => Some(&dff.reset),
            DffAny::Adff(dff) => Some(&dff.reset_n),
            DffAny::Dffe(_) => None,
            _ => None,
        }
    }

    pub fn enable_wire(&self) -> Option<&Wire<S>> {
        match self {
            DffAny::Sdffe(dff) => Some(&dff.en),
            DffAny::Adffe(dff) => Some(&dff.en),
            DffAny::Dffe(dff) => Some(&dff.en),
            _ => None,
        }
    }
}
