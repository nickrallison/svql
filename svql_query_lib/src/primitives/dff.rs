//! Primitive flip-flop definitions.
//!
//! This module provides specialized query components for various types of
//! flip-flops, including those with synchronous/asynchronous resets and
//! clock enables.

use svql_query::prelude::*;

// Match any flip-flop
svql_query::define_dff_primitive!(DffAny, [(clk, input), (d, input), (q, output)], |_| true);

// Flip-flop with synchronous reset and clock enable
svql_query::define_dff_primitive!(
    Sdffe,
    [
        (clk, input),
        (d, input),
        (reset, input),
        (en, input),
        (q, output)
    ],
    |cell| {
        if let prjunnamed_netlist::Cell::Dff(ff) = cell {
            ff.has_reset() && ff.has_enable()
        } else {
            false
        }
    }
);

// Flip-flop with asynchronous reset and clock enable
svql_query::define_dff_primitive!(
    Adffe,
    [
        (clk, input),
        (d, input),
        (reset_n, input),
        (en, input),
        (q, output)
    ],
    |cell| {
        if let prjunnamed_netlist::Cell::Dff(ff) = cell {
            ff.has_clear() && ff.has_enable()
        } else {
            false
        }
    }
);

// Flip-flop with synchronous reset, no enable
svql_query::define_dff_primitive!(
    Sdff,
    [(clk, input), (d, input), (reset, input), (q, output)],
    |cell| {
        if let prjunnamed_netlist::Cell::Dff(ff) = cell {
            ff.has_reset() && !ff.has_enable()
        } else {
            false
        }
    }
);

// Flip-flop with asynchronous reset, no enable
svql_query::define_dff_primitive!(
    Adff,
    [(clk, input), (d, input), (reset_n, input), (q, output)],
    |cell| {
        if let prjunnamed_netlist::Cell::Dff(ff) = cell {
            ff.has_clear() && !ff.has_enable()
        } else {
            false
        }
    }
);

// Flip-flop with clock enable, no reset
svql_query::define_dff_primitive!(
    Dffe,
    [(clk, input), (d, input), (en, input), (q, output)],
    |cell| {
        if let prjunnamed_netlist::Cell::Dff(ff) = cell {
            !ff.has_reset() && !ff.has_clear() && ff.has_enable()
        } else {
            false
        }
    }
);

// Basic flip-flop with no reset or enable
svql_query::define_dff_primitive!(Dff, [(clk, input), (d, input), (q, output)], |cell| {
    if let prjunnamed_netlist::Cell::Dff(ff) = cell {
        !ff.has_reset() && !ff.has_clear() && !ff.has_enable()
    } else {
        false
    }
});

#[cfg(test)]
mod tests {
    use super::*;
    use svql_query::query_test;

    query_test!(
        name: test_basic_dff,
        query: Dff,
        haystack: ("examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v", "and_q_double_sdffe"),
        expect: 2,
        config: |config_builder| config_builder
    );
}
