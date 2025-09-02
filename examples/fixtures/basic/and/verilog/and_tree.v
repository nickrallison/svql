module and_tree #(
    parameter N = 2
)
(
    input [0:N-1] x,
    output y
);

// Calculate the number of levels needed for the tree
localparam LEVELS = $clog2(N) + 1;

// Create arrays for each level of the tree
wire [0:N-1] level [0:LEVELS-1];

genvar i, j;

generate
    // Level 0: assign inputs
    for (i = 0; i < N; i = i + 1) begin : input_level
        assign level[0][i] = x[i];
    end
    
    // Generate tree levels
    for (i = 1; i < LEVELS; i = i + 1) begin : tree_levels
        localparam PREV_WIDTH = N >> (i-1);
        localparam CURR_WIDTH = (PREV_WIDTH + 1) >> 1;
        
        for (j = 0; j < CURR_WIDTH; j = j + 1) begin : level_nodes
            if (j * 2 + 1 < PREV_WIDTH) begin
                // Both inputs available
                assign level[i][j] = level[i-1][j*2] & level[i-1][j*2+1];
            end else if (j * 2 < PREV_WIDTH) begin
                // Only left input available (odd number case)
                assign level[i][j] = level[i-1][j*2];
            end
        end
    end
endgenerate

// Output is the final result
assign y = level[LEVELS-1][0];

endmodule