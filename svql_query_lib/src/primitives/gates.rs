//! Primitive hardware gate definitions.
//!
//! This module provides standard logic and arithmetic gates used as the
//! atomic building blocks for structural queries.

use svql_query::prelude::*;

// Logic gates
svql_query::define_primitive!(AndGate, And, [(a, input), (b, input), (y, output)]);

svql_query::define_primitive!(OrGate, Or, [(a, input), (b, input), (y, output)]);

svql_query::define_primitive!(NotGate, Not, [(a, input), (y, output)]);

svql_query::define_primitive!(BufGate, Buf, [(a, input), (y, output)]);

svql_query::define_primitive!(XorGate, Xor, [(a, input), (b, input), (y, output)]);

svql_query::define_primitive!(
    MuxGate,
    Mux,
    [(a, input), (b, input), (sel, input), (y, output)]
);

// Arithmetic gates
svql_query::define_primitive!(EqGate, Eq, [(a, input), (b, input), (y, output)]);

svql_query::define_primitive!(LtGate, ULt, [(a, input), (b, input), (y, output)]);

svql_query::define_primitive!(AddGate, Adc, [(a, input), (b, input), (y, output)]);

svql_query::define_primitive!(MulGate, Mul, [(a, input), (b, input), (y, output)]);

#[cfg(test)]
mod tests {
    use super::*;
    use svql_common::Dedupe;
    use svql_query::query_test;

    query_test!(
        name: test_and_gate,
        query: AndGate,
        haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
        expect: 3,
        config: |config_builder| config_builder.dedupe(Dedupe::All)
    );

    query_test!(
        name: test_or_gate,
        query: OrGate,
        haystack: ("examples/fixtures/basic/or/verilog/small_or_tree.v", "small_or_tree"),
        expect: 3,
        config: |config_builder| config_builder.dedupe(Dedupe::All)
    );
}
