// examples/fixtures/cwes/cwe1234/cwe1234_wide_reg.v
// Uses wider registers (32-bit) to test multi-bit DFF handling
// Pattern: write & (~lock | scan | debug)

module cwe1234_wide_reg (
    input [31:0] Data_in,
    input Clk,
    input resetn,
    input write,
    input Lock,
    input scan_mode,
    input debug_unlocked,
    output reg [31:0] Data_out
);

reg lock_status;

always @(posedge Clk or negedge resetn) begin
    if (~resetn) begin
        lock_status <= 1'b0;
    end else if (Lock) begin
        lock_status <= 1'b1;
    end else begin
        lock_status <= lock_status;
    end
end

// VULNERABILITY: 32-bit register with multiple bypass paths
always @(posedge Clk or negedge resetn) begin
    if (~resetn) begin
        Data_out <= 32'h00000000;
    end else if (write & (~lock_status | scan_mode | debug_unlocked)) begin
        Data_out <= Data_in;
    end else begin
        Data_out <= Data_out;
    end
end

endmodule