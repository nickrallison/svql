module and_mux
(
    input  a,
    input  b,
    output y
);

    // 2‑to‑1 multiplexer implementation of AND:
    //   y = a ? b : 1'b0;
    // The conditional operator (`?:`) is synthesised by Yosys as a MUX cell.
    assign y = a ? b : 1'b0;

endmodule