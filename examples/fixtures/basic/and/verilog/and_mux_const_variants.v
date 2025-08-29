// examples/fixtures/basic/and/verilog/and_mux_const_variants.v
// Contains one correct and_mux implementation and two incorrect constant variants.
// - y_good:     and via mux with 1'b0 on the false branch (matches pattern)
// - y_bad_one:  uses 1'b1 constant on the false branch (should not match)
// - y_bad_swap: uses 1'b0 on the true branch (arm swapped; should not match)

module and_mux_const_variants
(
    input  a,   input  b,
    input  a2,  input  b2,
    input  a3,  input  b3,
    output y_good,
    output y_bad_one,
    output y_bad_swap
);

    // Correct: y = a ? b : 1'b0
    assign y_good = a ? b : 1'b0;

    // Incorrect: constant is 1 instead of 0
    assign y_bad_one = a2 ? b2 : 1'b1;

    // Incorrect: constant is 0 but on the true branch (swapped)
    assign y_bad_swap = a3 ? 1'b0 : b3;

endmodule