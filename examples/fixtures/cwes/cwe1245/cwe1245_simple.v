// Simple 4-state FSM with unreachable state 3 (vulnerable).
module cwe1245_simple (
    input clk,
    input resetn,
    input cond_a, cond_b,
    output reg [3:0] state
);

always @(posedge clk or negedge resetn) begin
    if (~resetn) state <= 4'b0001;  // Reset to state 0 (ID 1 in one-hot)
    else case (state)
        4'b0001: state <= cond_a ? 4'b0010 : 4'b0001;  // State 0 -> 1 or self
        4'b0010: state <= cond_b ? 4'b0100 : 4'b0001;  // State 1 -> 2 or 0
        4'b0100: state <= 4'b0001;  // State 2 -> 0 (deadlock if no out, but has)
        // Missing: 4'b1000 (state 3 unreachable)
        default: state <= 4'b0001;
    endcase
end

endmodule
