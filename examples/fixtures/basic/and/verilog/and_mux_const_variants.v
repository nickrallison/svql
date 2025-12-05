
module and_mux_const_variants
(
    input  a,   input  b,
    input  a2,  input  b2,
    input  a3,  input  b3,
    output y_good,
    output y_bad_one,
    output y_bad_swap
);

    assign y_good = a ? b : 1'b0;

    assign y_bad_one = a2 ? b2 : 1'b1;

    assign y_bad_swap = a3 ? 1'b0 : b3;

endmodule