
module and_mux_const_bad
(
    input  a,  input  b,
    input  c,  input  d,
    output y_bad_one,
    output y_bad_swap
);

    assign y_bad_one = a ? b : 1'b1;

    assign y_bad_swap = c ? 1'b0 : d;

endmodule