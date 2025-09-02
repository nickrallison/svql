
module and_tree #(
    parameter N = 2,

    // Derived parameters
    parameter N1 = N / 2,
    parameter N2 = N - N1
)
(
    input [0:N-1] x,
    output y
);

genvar i;

generate
    if (N == 1) begin // base case, return input
        assign y = x[0];
    end else begin // recursive case
        wire y1;
        wire y2;
        wire [0:N1-1] x1;
        wire [0:N2-1] x2;

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
        ) and_tree_1 (
            .x(x1),
            .y(y1)
        );

        and_tree #(
            .N(N2),
        ) and_tree_2 (
            .x(x2),
            .y(y2)
        );

        // Combining Results
        assign y = y1 & y2;

    end
endgenerate
endmodule