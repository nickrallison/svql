//! Primitive flip-flop definitions.
//!
//! This module provides specialized query components for various types of
//! flip-flops, including those with synchronous/asynchronous resets and
//! clock enables.

use crate::common::{Config, ModuleConfig};
use crate::driver::{Context, Driver, DriverKey};
use crate::subgraph::cell::CellKind;
use crate::traits::{Hardware, Matched, Pattern};
use crate::{Instance, Match, ReportNode, Search, State, Wire};
use prjunnamed_netlist::Cell;

macro_rules! impl_dff_primitive {
    ($name:ident, [$($port:ident),*], $filter:expr, $description:expr) => {
        #[doc = $description]
        #[derive(Clone, Debug)]
        pub struct $name<S: State> {
            /// The hierarchical path of this flip-flop instance.
            pub path: Instance,
            $(
                #[doc = concat!("The ", stringify!($port), " port wire.")]
                pub $port: Wire<S>
            ),*
        }

        impl<S: State> $name<S> {
            $(
                #[doc = concat!("Returns a reference to the ", stringify!($port), " port.")]
                pub fn $port(&self) -> Option<&Wire<S>> {
                    Some(&self.$port)
                }
            )*
        }

        impl<S: State> Hardware for $name<S> {
            type State = S;

            fn path(&self) -> &Instance {
                &self.path
            }

            fn type_name(&self) -> &'static str {
                stringify!($name)
            }

            fn children(&self) -> Vec<&dyn Hardware<State = Self::State>> {
                vec![ $( &self.$port ),* ]
            }

            fn find_port(&self, path: &Instance) -> Option<&Wire<S>> {
                if !path.starts_with(self.path()) {
                    return None;
                }
                let rel_path = path.relative(self.path());
                let next = rel_path.first()?.as_ref();
                match next {
                    $(stringify!($port) => self.$port.find_port(path),)*
                    _ => None,
                }
            }

            fn report(&self, name: &str) -> ReportNode {
                let source_loc = [$(self.$port.source()),*]
                    .into_iter()
                    .flatten()
                    .next();

                ReportNode {
                    name: name.to_string(),
                    type_name: stringify!($name).to_string(),
                    path: self.path.clone(),
                    details: None,
                    source_loc,
                    children: Vec::new(),
                }
            }
        }

        impl Pattern for $name<Search> {
            type Match = $name<Match>;

            fn instantiate(base_path: Instance) -> Self {
                Self {
                    path: base_path.clone(),
                    $($port: Wire::new(base_path.child(stringify!($port)), ()),)*
                }
            }

            fn context(
                _driver: &Driver,
                _options: &ModuleConfig
            ) -> Result<Context, Box<dyn std::error::Error>> {
                Ok(Context::new())
            }

            fn execute(
                &self,
                _driver: &Driver,
                context: &Context,
                key: &DriverKey,
                _config: &Config
            ) -> Vec<Self::Match> {
                let haystack = context.get(key).expect("Haystack missing from context");
                let index = haystack.index();

                let matches: Vec<_> = index.cells_of_type_iter(CellKind::Dff)
                    .into_iter()
                    .flatten()
                    .filter(|cell_wrapper| {
                        match cell_wrapper.get() {
                            Cell::Dff(ff) => {
                                let check: fn(&prjunnamed_netlist::FlipFlop) -> bool = $filter;
                                check(ff)
                            }
                            _ => false,
                        }
                    })
                    .map(|cell| {
                        $name {
                            path: self.path.clone(),
                            $($port: Wire::new(self.$port.path.clone(), Some(cell.to_info()))),*
                        }
                    })

                    .collect();
                matches
            }
        }

        impl Matched for $name<Match> {
            type Search = $name<Search>;
        }
    };
}

impl_dff_primitive!(
    DffAny,
    [clk, d, en, q],
    |_| true,
    "Matches any flip-flop cell regardless of reset or enable configuration."
);

impl_dff_primitive!(
    Sdffe,
    [clk, d, reset, en, q],
    |ff| ff.has_reset() && ff.has_enable(),
    "Matches flip-flops with synchronous reset and clock enable."
);

impl_dff_primitive!(
    Adffe,
    [clk, d, reset_n, en, q],
    |ff| ff.has_clear() && ff.has_enable(),
    "Matches flip-flops with asynchronous reset (clear) and clock enable."
);

impl_dff_primitive!(
    Sdff,
    [clk, d, reset, q],
    |ff| ff.has_reset() && !ff.has_enable(),
    "Matches flip-flops with synchronous reset and no clock enable."
);

impl_dff_primitive!(
    Adff,
    [clk, d, reset_n, q],
    |ff| ff.has_clear() && !ff.has_enable(),
    "Matches flip-flops with asynchronous reset (clear) and no clock enable."
);

impl_dff_primitive!(
    Dffe,
    [clk, d, en, q],
    |ff| !ff.has_reset() && !ff.has_clear() && ff.has_enable(),
    "Matches flip-flops with clock enable and no reset logic."
);

impl_dff_primitive!(
    Dff,
    [clk, d, q],
    |ff| !ff.has_reset() && !ff.has_clear() && !ff.has_enable(),
    "Matches basic flip-flops with no reset or clock enable logic."
);
