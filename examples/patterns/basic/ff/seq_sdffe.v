
module seq_sdffe
(
input clk,
input d,
input reset,
output wire q
);

parameter FIFO_DEPTH = 8;

reg fifo [0:FIFO_DEPTH-1];

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