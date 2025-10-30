// examples/fixtures/security/access_control/locked_reg/verilog/cwe1234_deep.v
// Deep OR tree with 4 bypass conditions
// Pattern: write & (((~lock | scan) | debug) | test_mode)
// Tests recursive tree traversal at depth 3

module cwe1234_deep (
    input [15:0] Data_in,
    input Clk,
    input resetn,
    input write,
    input Lock,
    input scan_mode,
    input debug_unlocked,
    input test_mode,
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

// VULNERABILITY: Deep OR tree with multiple bypass paths
// Yosys will synthesize this as a tree of OR gates
always @(posedge Clk or negedge resetn)
if (~resetn) begin
    Data_out <= 16'h0000;
end
else if (write & (~lock_status | scan_mode | debug_unlocked | test_mode)) begin
    Data_out <= Data_in;
end
else begin
    Data_out <= Data_out;
end

endmodule