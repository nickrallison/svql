use svql_query::prelude::*;

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

    const PORTS: &'static [Port] = &[Port::input("a"), Port::input("b"), Port::output("y")];

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
    const SUBMODULES: &'static [Submodule] = &[
        Submodule::of::<AndGate>("and1"),
        Submodule::of::<AndGate>("and2"),
    ];

    const ALIASES: &'static [Alias] = &[
        Alias::input("a", Selector::static_path(&["and1", "a"])),
        Alias::input("b", Selector::static_path(&["and1", "b"])),
        Alias::output("y", Selector::static_path(&["and2", "y"])),
    ];

    const CONNECTIONS: Connections = {
        const CONN_GROUP: &[Connection] = &[
            // and1.y can connect to EITHER and2.a OR and2.b (commutative inputs)
            Connection::new(
                Selector::static_path(&["and1", "y"]),
                Selector::static_path(&["and2", "a"]),
            ),
            Connection::new(
                Selector::static_path(&["and1", "y"]),
                Selector::static_path(&["and2", "b"]),
            ),
        ];

        Connections {
            connections: &[CONN_GROUP],
        }
    };

    const DEPENDANCIES: &'static [&'static ExecInfo] = &[<AndGate as Pattern>::EXEC_INFO];

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
    let haystack_path = "examples/fixtures/basic/and/verilog/small_and_tree.v";
    let haystack_module = "small_and_tree";
    let config = Config::builder().dedupe(Dedupe::None).build();

    let driver = Driver::new_workspace()?;

    let key = DriverKey::new(haystack_path, haystack_module);
    let container = driver
        .get_design(&key, &config.haystack_options)
        .map_err(|e| QueryError::design_load(e.to_string()))?;

    for cell in container.index().cells_topo() {
        println!("Cell: {:#?}", cell);
    }

    // Execute query using the new DataFrame API
    let store = svql_query::run_query::<And2Gates>(&driver, &key, &config)?;

    for (_, table) in store.tables() {
        println!("Table: {}", table);
    }

    // Get the result count from the store
    // let results_table = store.get::<And2Gates>().expect("Table should be present");
    // let rows = results_table.rows().collect::<Vec<_>>();
    // let stored_count = rows.len();
    Ok(())
}
