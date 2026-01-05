use common::{Config, Dedupe, MatchLength};
use driver::Driver;
use svql_macros::{composite, netlist, variant};

use svql_query::prelude::*;

use tracing::{Level, info};

// ============================================================================
// 1. Define Netlists
// ============================================================================

#[netlist(
    file = "examples/patterns/basic/and/verilog/and_gate.v",
    name = "and_gate"
)]
pub struct AndGate<S: State> {
    pub a: Wire<S>,
    pub b: Wire<S>,
    pub y: Wire<S>,
}

#[netlist(
    file = "examples/patterns/basic/or/verilog/or_gate.v",
    name = "or_gate"
)]
pub struct OrGate<S: State> {
    pub a: Wire<S>,
    pub b: Wire<S>,
    pub y: Wire<S>,
}

#[netlist(file = "examples/patterns/basic/ff/rtlil/sdffe.il", name = "sdffe")]
pub struct Dff<S: State> {
    pub clk: Wire<S>,
    pub d: Wire<S>,
    pub q: Wire<S>,
}

// ============================================================================
// 2. Define Variant
// ============================================================================

#[variant(ports(y))]
pub enum LogicGate<S: State> {
    #[variant(map(y = "y"))]
    And(AndGate<S>),

    #[variant(map(y = "y"))]
    Or(OrGate<S>),
}

// ============================================================================
// 3. Define Composite
// ============================================================================

#[composite]
pub struct RegisteredLogic<S: State> {
    #[path]
    pub path: Instance,

    #[submodule]
    pub logic: LogicGate<S>,

    #[submodule]
    pub reg: Dff<S>,
}

impl<S: State> Topology<S> for RegisteredLogic<S> {
    fn define_connections<'a>(&'a self, ctx: &mut ConnectionBuilder<'a, S>) {
        ctx.connect(self.logic.y(), Some(&self.reg.d));
    }
}

// ============================================================================
// 4. Main Execution
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup Logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // 2. Setup Driver & Config
    let config = Config::builder()
        .match_length(MatchLength::NeedleSubsetHaystack)
        .dedupe(Dedupe::None)
        .build();

    let driver = Driver::new_workspace()?;

    // 3. Load Haystack
    let design_path = "examples/fixtures/larger_designs/json/openpiton_tile_flat.json";
    let design_module = "tile";

    info!("Loading design...");
    // Handle potential error if file doesn't exist (for demo purposes)
    let (haystack_key, haystack_design) =
        match driver.get_or_load_design_raw(design_path, design_module) {
            Ok(res) => res,
            Err(e) => {
                info!("Skipping demo execution: {}", e);
                return Ok(());
            }
        };

    // 4. Build Context
    info!("Building context...");
    let ctx_and = AndGate::<Search>::context(&driver, &config.needle_options)?;
    let ctx_or = OrGate::<Search>::context(&driver, &config.needle_options)?;
    let ctx_dff = Dff::<Search>::context(&driver, &config.needle_options)?;

    let mut context = ctx_and.merge(ctx_or).merge(ctx_dff);
    context = context.with_design(haystack_key.clone(), haystack_design);

    // 5. Instantiate Query
    info!("Instantiating query...");
    let query_root = Instance::root("my_query".to_string());
    let query = RegisteredLogic::<Search>::instantiate(query_root);

    // 6. Execute
    info!("Executing query...");
    let results = query.query(&driver, &context, &haystack_key, &config);

    info!("Found {} matches", results.len());

    // 7. Inspect
    for (i, match_inst) in results.iter().enumerate() {
        info!("Match #{}:", i);

        match &match_inst.logic {
            LogicGate::And(and_gate) => {
                info!("  Logic: AND, Output: {:?}", and_gate.y.inner);
            }
            LogicGate::Or(or_gate) => {
                info!("  Logic: OR, Output: {:?}", or_gate.y.inner);
            }
            _ => unreachable!(),
        }

        info!("  Reg Q: {:?}", match_inst.reg.q.inner);
    }

    Ok(())
}
