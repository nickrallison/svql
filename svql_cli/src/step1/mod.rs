pub use svql_query_lib::gates::AdcGate;

use svql_query::prelude::*;

/// Example composite component representing an ADC gate with carry-out.
#[derive(Debug, Clone)]
pub struct AdcWithCarry {
    /// Input A
    pub a: Wire,
    /// Input B
    pub b: Wire,
    /// Sum output (A XOR B)
    pub sum: Wire,
    /// Carry output (A AND B)
    pub carry: Wire,
}

impl Component for AdcWithCarry {
    type Kind = kind::Primitive;
}

impl Primitive for AdcWithCarry {
    const CELL_KIND: CellKind = CellKind::Adc;
    const PORTS: &'static [PortDecl] = &[
        PortDecl::input("a"),
        PortDecl::input("b"),
        PortDecl::output("sum"),
        PortDecl::output("carry"),
    ];

    fn resolve(wrapper: &CellWrapper<'_>) -> EntryArray {
        let cell = wrapper.get();
        println!("Resolving cell: {:?}", cell);
        let y = wrapper.output_wire();
        let size_y = y.len();
        let mut entries = vec![ColumnEntry::Null; 4];
        entries[0] = wrapper
            .input_wire("a")
            .map(ColumnEntry::Wire)
            .unwrap_or(ColumnEntry::Null);
        entries[1] = wrapper
            .input_wire("b")
            .map(ColumnEntry::Wire)
            .unwrap_or(ColumnEntry::Null);
        entries[2] = ColumnEntry::Wire(y.slice(0..size_y - 1));
        entries[3] = ColumnEntry::Wire(y.slice(size_y - 1..size_y));
        EntryArray::new(entries)
    }

    fn primitive_rehydrate(
        row: &Row<Self>,
        _store: &Store,
        _driver: &Driver,
        _key: &DriverKey,
        _config: &svql_common::Config,
    ) -> Option<Self> {
        Some(Self {
            a: row.wire("a")?.clone(),
            b: row.wire("b")?.clone(),
            sum: row.wire("sum")?.clone(),
            carry: row.wire("carry")?.clone(),
        })
    }
}
