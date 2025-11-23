// svql_query/src/variants/dff_any.rs
use crate::primitives::dff::{Adff, Adffe, Dffe, Sdff, Sdffe};
use crate::{State, Wire};
use svql_macros::variant;

variant! {
    name: DffAny,
    variants: [
        (Sdffe, "sdffe", Sdffe),
        (Adffe, "adffe", Adffe),
        (Sdff, "sdff", Sdff),
        (Adff, "adff", Adff),
        (Dffe, "dffe", Dffe),
    ],
    common_ports: {
        clk: "clock",
        d: "data_input",
        q: "output"
    }
}

impl<S> DffAny<S>
where
    S: State,
{
    pub fn dff_type(&self) -> &'static str {
        match self {
            DffAny::Sdffe(_) => "Sync Reset Enable DFF",
            DffAny::Adffe(_) => "Async Reset Enable DFF",
            DffAny::Sdff(_) => "Sync Reset DFF",
            DffAny::Adff(_) => "Async Reset DFF",
            DffAny::Dffe(_) => "Enable DFF",
        }
    }

    pub fn reset_wire(&self) -> Option<&Wire<S>> {
        match self {
            DffAny::Sdffe(dff) => Some(&dff.reset),
            DffAny::Adffe(dff) => Some(&dff.reset_n),
            DffAny::Sdff(dff) => Some(&dff.reset),
            DffAny::Adff(dff) => Some(&dff.reset_n),
            DffAny::Dffe(_) => None,
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
    // NEW: Dummy new for compatibility as composite sub (uses first variant)
    pub fn new(path: crate::instance::Instance) -> Self {
        // Use SyncReset variant as dummy for search-time construction
        // Inner path uses the variant's inst_name
        DffAny::Sdffe(Sdffe::new(path.child("sdffe".to_string())))
    }
}
