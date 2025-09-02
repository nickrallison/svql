
module and_seq #(
    parameter N = 2,
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
        wire [0:N-2] x1;
        wire x2;

        // Splitting Inputs
        genvar j;
        for (j = 0; j < N - 1; j = j + 1) begin
            assign x1[j] = x[j];
        end

        assign x2 = x[N - 1];
    

        // Recursive Work
        and_seq #(
            .N(N-1)
        ) and_seq_1 (
            .x(x1),
            .y(y1)
        );

        // Combining Results
        assign y = y1 & x2;

    end
endgenerate
endmodule