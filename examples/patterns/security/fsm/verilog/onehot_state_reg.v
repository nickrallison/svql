// Dummy one-hot state reg.
module onehot_state_reg (
    input clk,
    input resetn,
    input [3:0] next_state,
    output reg [3:0] state
);

always @(posedge clk or negedge resetn) begin
    if (!resetn) state <= 4'b0001;
    else state <= next_state;
end

endmodule
