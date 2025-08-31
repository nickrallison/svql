parameter FIFO_WIDTH = 2;
parameter FIFO_DEPTH = 2;

module seq_2_width_2_sdffe
(
input clk,
input [FIFO_WIDTH-1:0] d,
input reset,
output wire [FIFO_WIDTH-1:0] q
);

reg [FIFO_WIDTH-1:0] fifo [0:FIFO_DEPTH-1];

always @(posedge clk) begin
    if (reset) fifo[0] <= 1'b0;
    else fifo[0] <= d;
end

genvar i;
for (i = 1; i < FIFO_DEPTH; i = i + 1) begin
    always @(posedge clk) begin
        if (reset) fifo[i] <= 1'b0;
        else fifo[i] <= fifo[i-1];
    end
end

assign q = fifo[FIFO_DEPTH-1];

endmodule