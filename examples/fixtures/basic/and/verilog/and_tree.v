
module and_tree #(
    parameter N = 2,
    parameter WIDTH = 1
)

(
input [N-1:0] x [WIDTH-1:0],
output y [WIDTH-1:0]
);

parameter N1 = N / 2;
parameter N2 = N - N1;

genvar i;

generate
    if (N == 1) begin // base case, return input
        assign y = x[0];
    end else begin // recursive case
        wire y1 [WIDTH-1:0];
        wire y2 [WIDTH-1:0];
        wire x1 [N1-1:0][WIDTH-1:0];
        wire x2 [N2-1:0][WIDTH-1:0];

        assign x1 = x[N1-1:0];
        assign x2 = x[N-1:N1];

        and_tree #(
            .N(N1),
            .WIDTH(WIDTH)
        ) and_tree_1 (
            .x(x1),
            .y(y1)
        );

        and_tree #(
            .N(N2),
            .WIDTH(WIDTH)
        ) and_tree_2 (
            .x(x2),
            .y(y2)
        );

        assign y = y1 & y2;
    end
endgenerate
endmodule