use svql_common::Dedupe;
use svql_query::{
    Wire,
    prelude::*,
    selector::Selector,
    session::{ExecInfo, Row, Store},
    test_harness::TestSpec,
    traits::{
        Component, Netlist, PatternInternal,
        composite::{Composite, Connection, Connections, Endpoint},
        kind,
    },
};

#[allow(unused)]
#[derive(Debug, Clone)]
struct AndGate {
    a: Wire,
    b: Wire,
    y: Wire,
}

impl Netlist for AndGate {
    const MODULE_NAME: &'static str = "and_gate";
    const FILE_PATH: &'static str = "examples/fixtures/basic/and/verilog/and_gate.v";
    const DEFS: &'static [ColumnDef] = &[
        ColumnDef::new("a", ColumnKind::Cell, false),
        ColumnDef::new("b", ColumnKind::Cell, false),
        ColumnDef::new("y", ColumnKind::Cell, false),
    ];

    fn schema() -> &'static svql_query::session::PatternSchema {
        static INSTANCE: std::sync::OnceLock<svql_query::session::PatternSchema> =
            std::sync::OnceLock::new();
        INSTANCE.get_or_init(|| svql_query::session::PatternSchema::new(<Self as Netlist>::DEFS))
    }

    fn rehydrate<'a>(
        row: &Row<Self>,
        _store: &Store,
        _driver: &Driver,
        _key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Netlist> + Send + Sync + 'static,
    {
        let a_id = row.wire("a")?;
        let b_id = row.wire("b")?;
        let y_id = row.wire("y")?;

        let and_gate = AndGate {
            a: a_id,
            b: b_id,
            y: y_id,
        };

        Some(and_gate)
    }
}

impl Component for AndGate {
    type Kind = kind::Netlist;
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct And2Gates {
    and1: AndGate,
    and2: AndGate,
}

impl Composite for And2Gates {
    const DEFS: &'static [ColumnDef] = &[
        ColumnDef::sub::<AndGate>("and1"),
        ColumnDef::sub::<AndGate>("and2"),
    ];

    const ALIASES: &'static [(&'static str, Endpoint)] = &[];

    fn schema() -> &'static svql_query::session::PatternSchema {
        static INSTANCE: std::sync::OnceLock<svql_query::session::PatternSchema> =
            std::sync::OnceLock::new();
        INSTANCE.get_or_init(|| svql_query::session::PatternSchema::new(<Self as Composite>::DEFS))
    }

    fn rehydrate<'a>(
        _row: &Row<Self>,
        _store: &Store,
        _driver: &Driver,
        _key: &DriverKey,
    ) -> Option<Self>
    where
        Self: Component + PatternInternal<kind::Composite> + Send + Sync + 'static,
    {
        todo!()
    }

    const CONNECTIONS: Connections = {
        let conns: &'static [&'static [Connection]] = &[&[
            // Connect and1.y -> and2.a
            Connection {
                from: Endpoint {
                    selector: Selector::new(&["and1", "y"]),
                },
                to: Endpoint {
                    selector: Selector::new(&["and2", "a"]),
                },
            },
            Connection {
                from: Endpoint {
                    selector: Selector::new(&["and1", "y"]),
                },
                to: Endpoint {
                    selector: Selector::new(&["and2", "b"]),
                },
            },
        ]];
        Connections { connections: conns }
    };

    const DEPENDANCIES: &'static [&'static ExecInfo] = &[<AndGate as Pattern>::EXEC_INFO];

    fn preload_driver(
        driver: &Driver,
        design_key: &DriverKey,
        config: &svql_common::Config,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        <AndGate as Pattern>::preload_driver(driver, design_key, config)
    }
}

impl Component for And2Gates {
    type Kind = kind::Composite;
}

// query_test!(
//     name: test_and2gates_small_and_tree_dedupe_none,
//     query: And2Gates,
//     haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
//     expect: 4,
//     config: |config_builder| config_builder.dedupe(Dedupe::None)
// );

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let spec = TestSpec {
        haystack_path: "examples/fixtures/basic/and/verilog/small_and_tree.v",
        haystack_module: "small_and_tree",
        expected_count: 4,
        config_fn: Some(|config_builder| config_builder.dedupe(Dedupe::None)),
    };

    let driver = Driver::new_workspace()?;

    let mut config_builder = Config::builder();
    if let Some(f) = spec.config_fn {
        config_builder = f(config_builder);
    }
    let config = config_builder.build();

    let container = spec.get_design(&driver, &config)?;

    for cell in container.index().cells_topo() {
        println!("Cell: {:#?}", cell);
    }

    // Execute query using the new DataFrame API
    let store = svql_query::run_query::<And2Gates>(&driver, &spec.get_key(), &config)?;

    for (_, table) in store.tables() {
        println!("Table: {}", table);
    }

    // Get the result count from the store
    // let results_table = store.get::<And2Gates>().expect("Table should be present");
    // let rows = results_table.rows().collect::<Vec<_>>();
    // let stored_count = rows.len();
    Ok(())
}
