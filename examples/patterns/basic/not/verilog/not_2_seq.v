module not_2_seq
(
    input  a,
    output y
);
    wire inner;
    assign inner = ~a;

    assign y = ~inner;

endmodule