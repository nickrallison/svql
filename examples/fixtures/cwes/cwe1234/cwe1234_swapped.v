// examples/fixtures/security/access_control/locked_reg/verilog/cwe1234_swapped.v
// Tests commutative input handling - bypass conditions in different positions
// Pattern: (scan | ~lock) & write  (OR first, then AND)

module cwe1234_swapped (
    input [15:0] Data_in,
    input Clk,
    input resetn,
    input write,
    input Lock,
    input scan_mode,
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

// VULNERABILITY: Bypass conditions come first (tests commutative AND matching)
// Pattern: (~lock_status | scan_mode | debug_unlocked) & write
always @(posedge Clk or negedge resetn)
if (~resetn) begin
    Data_out <= 16'h0000;
end
else if ((~lock_status | scan_mode | debug_unlocked) & write) begin
    Data_out <= Data_in;
end
else begin
    Data_out <= Data_out;
end

endmodule