// examples/fixtures/basic/and/verilog/and_nor_tree.v
// 8-input AND implemented entirely with and_nor submodules.
// Structure: 4 pairwise -> 2 -> 1 (total 7 and_nor instances)



module and_nor_tree
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

    // Stage 1
    and_nor u_and_nor_0 (
        .a(a),
        .b(b),
        .y(s0)
    );

    and_nor u_and_nor_1 (
        .a(c),
        .b(d),
        .y(s1)
    );

    and_nor u_and_nor_2 (
        .a(e),
        .b(f),
        .y(s2)
    );

    and_nor u_and_nor_3 (
        .a(g),
        .b(h),
        .y(s3)
    );

    // Stage 2
    and_nor u_and_nor_4 (
        .a(s0),
        .b(s1),
        .y(t0)
    );

    and_nor u_and_nor_5 (
        .a(s2),
        .b(s3),
        .y(t1)
    );

    // Stage 3
    and_nor u_and_nor_6 (
        .a(t0),
        .b(t1),
        .y(y)
    );

endmodule

module and_nor
(
    input  a,
    input  b,
    output y
);

wire nor_out;
wire not_a, not_b;

assign not_a = ~a;
assign not_b = ~b;

assign nor_out = not_a | not_b;

assign y = ~nor_out;

endmodule