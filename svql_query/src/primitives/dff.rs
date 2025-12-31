use crate::svql_common::{Config, ModuleConfig};
use crate::svql_driver::{Context, Driver, DriverKey};
use crate::svql_subgraph::cell::CellKind;
use crate::traits::{Component, PlannedQuery, Query, Reportable, Searchable};
use crate::{Instance, Match, Search, State, Wire};
use prjunnamed_netlist::{Cell, ControlNet};
use std::sync::Arc;
use tracing::debug;

/// Helper to determine if a DFF control port is actually being used.
fn is_active(net: &ControlNet) -> bool {
    match net {
        // If tied to a constant that never triggers, it is inactive.
        // Note: This logic assumes standard Yosys mapping where
        // inactive enables are tied to 1 and inactive resets to 0.
        ControlNet::Pos(n) => !n.is_const(),
        ControlNet::Neg(n) => !n.is_const(),
    }
}

macro_rules! impl_dff_primitive {
    ($name:ident, [$($port:ident),*], $filter:expr) => {
        #[derive(Clone, Debug)]
        pub struct $name<S: State> {
            pub path: Instance,
            $(pub $port: Wire<S>),*
        }

        impl<S: State> $name<S> {
            $(
                pub fn $port(&self) -> Option<&Wire<S>> {
                    Some(&self.$port)
                }
            )*
        }


        impl<S: State> Component<S> for $name<S> {
            fn path(&self) -> &Instance { &self.path }
            fn type_name(&self) -> &'static str { stringify!($name) }
            fn find_port(&self, path: &Instance) -> Option<&Wire<S>> {
                if !path.starts_with(self.path()) { return None; }
                self.find_port_inner(path.relative(self.path()))
            }
            fn find_port_inner(&self, rel_path: &[Arc<str>]) -> Option<&Wire<S>> {
                let next = rel_path.first()?.as_ref();
                let tail = &rel_path[1..];
                match next {
                    $(stringify!($port) => self.$port.find_port_inner(tail),)*
                    _ => None,
                }
            }
        }

        impl Searchable for $name<Search> {
            fn instantiate(base_path: Instance) -> Self {
                Self {
                    path: base_path.clone(),
                    $($port: Wire::new(base_path.child(stringify!($port)), ()),)*
                }
            }
        }

        impl<'a> Reportable for $name<Match<'a>> {
            fn to_report(&self, name: &str) -> crate::report::ReportNode {
                use crate::svql_subgraph::cell::SourceLocation;
                let source_loc = [$(self.$port.inner.get_source()),*]
                    .into_iter().flatten().next().unwrap_or_else(|| {
                        SourceLocation { file: Arc::from(""), lines: Vec::new() }
                    });
                crate::report::ReportNode {
                    name: name.to_string(),
                    type_name: stringify!($name).to_string(),
                    path: self.path.clone(),
                    details: None,
                    source_loc,
                    children: Vec::new(),
                }
            }
        }

        impl Query for $name<Search> {
            type Matched<'a> = $name<Match<'a>>;
            fn query<'a>(
                &self,
                _driver: &Driver,
                context: &'a Context,
                key: &DriverKey,
                _config: &Config
            ) -> Vec<Self::Matched<'a>> {
                let haystack = context.get(key).expect("Haystack missing");
                let index = haystack.index();

                let real = index.cells_of_type_iter(CellKind::Dff)
                    .into_iter()
                    .flatten()
                    .filter(|cell_wrapper| {
                        if let Cell::Dff(ff) = cell_wrapper.get() {
                            let check: fn(&prjunnamed_netlist::FlipFlop) -> bool = $filter;
                            check(ff)
                        } else {
                            false
                        }
                    })
                    .map(|cell| {
                        $name {
                            path: self.path.clone(),
                            $($port: Wire::new(self.$port.path.clone(), cell.clone())),*
                        }
                    })
                    .collect();

                // debugging
                let cells_of_type: Vec<_> = index.cells_of_type_iter(CellKind::Dff)
                    .into_iter()
                    .flatten()
                    .collect();

                debug!("Total Dff cells found: {}", cells_of_type.len());

                let filtered: Vec<_> = cells_of_type.iter()
                    .filter(|cell_wrapper| {
                        if let Cell::Dff(ff) = cell_wrapper.get() {
                            let check: fn(&prjunnamed_netlist::FlipFlop) -> bool = $filter;
                            check(ff)
                        } else {
                            false
                        }
                    })
                    .collect();
                debug!("Filtered Dff cells found: {}", filtered.len());

                let mapped: Vec<_> = filtered.iter()
                    .map(|cell| {
                        Self::Matched {
                            path: self.path.clone(),
                            $($port: Wire::new(self.$port.path.clone(), cell.clone().clone().clone())),*
                        }
                    }).collect();
                debug!("Mapped Dff cells found: {}", mapped.len());

                return real;
            }
            fn context(_d: &Driver, _o: &ModuleConfig) -> Result<Context, Box<dyn std::error::Error>> {
                Ok(Context::new())
            }
        }

        impl PlannedQuery for $name<Search> {
            fn expected_schema(&self) -> crate::ir::Schema {
                crate::ir::Schema { columns: vec![$(stringify!($port).to_string()),*] }
            }
            fn get_column_index(&self, rel_path: &[Arc<str>]) -> Option<usize> {
                let next = rel_path.first()?.as_ref();
                let mut i = 0;
                $( if next == stringify!($port) { return Some(i); } i += 1; )*
                None
            }
            fn reconstruct<'a>(&self, cursor: &mut crate::ir::ResultCursor<'a>) -> Self::Matched<'a> {
                $name {
                    path: self.path.clone(),
                    $($port: Wire::new(self.$port.path.clone(), cursor.next_cell())),*
                }
            }
        }
    };
}

// Any Dff: No filtering
impl_dff_primitive!(DffAny, [clk, d, en, q], |_| true);

fn sdffe_filter(ff: &prjunnamed_netlist::FlipFlop) -> bool {
    // ff.has_reset() && ff.has_enable()
    let has_rst = ff.has_reset();
    let has_en = ff.has_enable();
    debug!(
        "Checking Sdffe: {:#?} filter: has_reset={} has_enable={}",
        ff, has_rst, has_en,
    );
    has_rst && has_en
}

// Sdffe: Synchronous Reset AND Enable
impl_dff_primitive!(Sdffe, [clk, d, reset, en, q], sdffe_filter);

// Adffe: Asynchronous Reset (Clear) AND Enable
impl_dff_primitive!(Adffe, [clk, d, reset_n, en, q], |ff| {
    // ff.has_clear() && ff.has_enable()
    let has_clr = ff.has_clear();
    let has_en = ff.has_enable();
    debug!(
        "Checking Adffe: {:#?} filter: has_clear={} has_enable={}",
        ff, has_clr, has_en,
    );
    has_clr && has_en
});

// Sdff: Synchronous Reset, NO Enable
impl_dff_primitive!(Sdff, [clk, d, reset, q], |ff| {
    // ff.has_reset() && !ff.has_enable()
    let has_rst = ff.has_reset();
    let has_en = ff.has_enable();
    debug!(
        "Checking Sdff: {:#?} filter: has_reset={} has_enable={}",
        ff, has_rst, has_en,
    );
    has_rst && !has_en
});

// Adff: Asynchronous Reset (Clear), NO Enable
impl_dff_primitive!(Adff, [clk, d, reset_n, q], |ff| {
    // ff.has_clear() && !ff.has_enable()
    let has_clr = ff.has_clear();
    let has_en = ff.has_enable();
    debug!(
        "Checking Adff: {:#?} filter: has_clear={} has_enable={}",
        ff, has_clr, has_en,
    );
    has_clr && !has_en
});

// Dffe: Enable, NO Reset (Sync or Async)
impl_dff_primitive!(Dffe, [clk, d, en, q], |ff| {
    // !ff.has_reset() && !ff.has_clear() && ff.has_enable()
    let has_rst = ff.has_reset();
    let has_clr = ff.has_clear();
    let has_en = ff.has_enable();
    debug!(
        "Checking Dffe: {:#?} filter: has_reset={} has_clear={} has_enable={}",
        ff, has_rst, has_clr, has_en,
    );
    !has_rst && !has_clr && has_en
});
