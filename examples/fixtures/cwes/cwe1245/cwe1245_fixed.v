// Complete 4-state FSM (all reachable, no gaps).
module cwe1245_fixed (
    input clk,
    input resetn,
    input cond_a, cond_b,
    output reg [3:0] state
);

always @(posedge clk or negedge resetn) begin
    if (~resetn) state <= 4'b0001;
    else case (state)
        4'b0001: state <= cond_a ? 4'b0010 : 4'b0001;
        4'b0010: state <= cond_b ? 4'b0100 : 4'b0001;
        4'b0100: state <= 4'b1000;  // To state 3
        4'b1000: state <= 4'b0001;  // Full cycle, no gaps
        default: state <= 4'b0001;
    endcase
end

endmodule
