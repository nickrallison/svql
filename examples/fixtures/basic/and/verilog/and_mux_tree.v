
module and_mux_tree
(
    input  a,
    input  b,
    input  c,
    input  d,
    input  e,
    input  f,
    input  g,
    input  h,
    output y
);

    wire s0, s1, s2, s3;
    wire t0, t1;

    and_mux u_and_mux_0 (
        .a(a),
        .b(b),
        .y(s0)
    );

    and_mux u_and_mux_1 (
        .a(c),
        .b(d),
        .y(s1)
    );

    and_mux u_and_mux_2 (
        .a(e),
        .b(f),
        .y(s2)
    );

    and_mux u_and_mux_3 (
        .a(g),
        .b(h),
        .y(s3)
    );

    and_mux u_and_mux_4 (
        .a(s0),
        .b(s1),
        .y(t0)
    );

    and_mux u_and_mux_5 (
        .a(s2),
        .b(s3),
        .y(t1)
    );

    and_mux u_and_mux_6 (
        .a(t0),
        .b(t1),
        .y(y)
    );

endmodule

module and_mux
(
    input  a,
    input  b,
    output y
);

    assign y = a ? b : 1'b0;

endmodule