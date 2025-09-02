
module and_seq #(
    parameter N = 2,
    parameter WIDTH = 1,
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
        wire [WIDTH-1:0] x1 [0:N-2];
        wire [WIDTH-1:0] x2;

        // Splitting Inputs
        genvar j;
        for (j = 0; j < N - 1; j = j + 1) begin
            assign x1[j] = x[j];
        end

        assign x2 = x[N - 1];
    

        // Recursive Work
        and_seq #(
            .N(N1),
            .WIDTH(WIDTH)
        ) and_seq_1 (
            .x(x1),
            .y(y1)
        );

        // Combining Results
        genvar k;
        for (k = 0; k < WIDTH; k = k + 1) begin
            assign y[k] = y1[k] & x2[k];
        end
    end
endgenerate
endmodule