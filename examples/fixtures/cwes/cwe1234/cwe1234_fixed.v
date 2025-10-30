// examples/fixtures/security/access_control/locked_reg/verilog/cwe1234_fixed.v
// FIXED version - no bypass vulnerability
// Lock cannot be overridden once set

module cwe1234_fixed (
    input [15:0] Data_in,
    input Clk,
    input resetn,
    input write,
    input Lock,
    input scan_mode,        // Present but NOT used to bypass lock
    input debug_unlocked,   // Present but NOT used to bypass lock
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

// FIXED: No bypass - lock cannot be overridden
// Only unlock path is through reset
always @(posedge Clk or negedge resetn)
if (~resetn) begin
    Data_out <= 16'h0000;
end
else if (write & ~lock_status) begin  // NO bypass conditions!
    Data_out <= Data_in;
end
else begin
    Data_out <= Data_out;
end

endmodule