// examples/fixtures/security/access_control/locked_reg/verilog/cwe1234_combined.v
// More complex bypass logic with AND and OR combinations
// Pattern: write & ((~lock & mode_a) | debug)

module cwe1234_combined (
    input [15:0] Data_in,
    input Clk,
    input resetn,
    input write,
    input Lock,
    input mode_a,
    input debug_unlocked,
    output reg [15:0] Data_out
);

reg lock_status;

always @(posedge Clk or negedge resetn)
if (~resetn) begin
    lock_status <= 1'b0;
end
else if (Lock) begin
    lock_status <= 1'b1;
end
else begin
    lock_status <= lock_status;
end

// VULNERABILITY: Combined AND/OR logic
// Pattern: write & ((~lock_status & mode_a) | debug_unlocked)
// Still has the core pattern: negated lock can be bypassed
always @(posedge Clk or negedge resetn)
if (~resetn) begin
    Data_out <= 16'h0000;
end
else if (write & ((~lock_status & mode_a) | debug_unlocked)) begin
    Data_out <= Data_in;
end
else begin
    Data_out <= Data_out;
end

endmodule