module and_seq #(
    parameter N = 2
)
(
    input [0:N-1] x,
    output y
);

genvar i;
wire [0:N-1] intermediate;

generate
    // Base case: first intermediate result is just the first input
    assign intermediate[0] = x[0];
    
    // Iteratively AND each subsequent input with the accumulated result
    for (i = 1; i < N; i = i + 1) begin : and_chain
        assign intermediate[i] = intermediate[i-1] & x[i];
    end
endgenerate

// Final output is the last intermediate result
assign y = intermediate[N-1];

endmodule