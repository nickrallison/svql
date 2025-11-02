// Two FSMs: one complete, one with deadlock (state 2 no out).
module cwe1245_multi (
    input clk, resetn,
    input cond,
    output reg [3:0] state1, state2  // state1: complete; state2: deadlock
);

always @(posedge clk or negedge resetn) begin
    if (~resetn) state1 <= 4'b0001;
    else if (cond) state1 <= 4'b0010; else state1 <= 4'b0001;  // Simple, complete
end

always @(posedge clk or negedge resetn) begin
    if (~resetn) state2 <= 4'b0001;
    else case (state2)
        4'b0001: state2 <= 4'b0010;
        4'b0010: state2 <= 4'b0100;  // To deadlock state
        4'b0100: ;  // Deadlock: no assignment
        default: state2 <= 4'b0001;
    endcase
end

endmodule
