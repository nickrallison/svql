
module and_tree #(
    parameter N = 2,
    parameter WIDTH = 1,

    // Derived parameters
    parameter N1 = N / 2,
    parameter N2 = N - N1
)
(
    input [WIDTH-1:0] x [0:N-1],
    output [WIDTH-1:0] y
);

genvar i;

generate
    if (N == 1) begin // base case, return input
        assign y = x[0];
    end else begin // recursive case
        wire [WIDTH-1:0] y1;
        wire [WIDTH-1:0] y2;
        wire [WIDTH-1:0] x1 [0:N1-1];
        wire [WIDTH-1:0] x2 [0:N2-1];

        // Splitting Inputs
        genvar j;
        for (j = 0; j < N1; j = j + 1) begin
            assign x1[j] = x[j];
        end
        
        for (j = 0; j < N2; j = j + 1) begin
            assign x2[j] = x[N1 + j];
        end

        // Recursive Work
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

        // Combining Results
        genvar k;
        for (k = 0; k < WIDTH; k = k + 1) begin
            assign y[k] = y1[k] & y2[k];
        end
    end
endgenerate
endmodule