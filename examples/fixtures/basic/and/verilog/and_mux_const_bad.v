// examples/fixtures/basic/and/verilog/and_mux_const_bad.v
// Contains only incorrect and_mux variants; expected to produce 0 matches.
// - y_bad_one:  uses 1'b1 constant on the false branch
// - y_bad_swap: uses 1'b0 on the true branch

module and_mux_const_bad
(
    input  a,  input  b,
    input  c,  input  d,
    output y_bad_one,
    output y_bad_swap
);

    // Incorrect: constant is 1 instead of 0
    assign y_bad_one = a ? b : 1'b1;

    // Incorrect: constant is 0 but on the true branch (swapped)
    assign y_bad_swap = c ? 1'b0 : d;

endmodule