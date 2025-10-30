// examples/fixtures/security/access_control/locked_reg/verilog/cwe1234_multi_reg.v
// Multiple vulnerable registers with different bypass combinations
// Tests that query finds all vulnerable patterns in one module

module cwe1234_multi_reg (
    input [15:0] Data_in_1,
    input [15:0] Data_in_2,
    input [31:0] Data_in_3,
    input Clk,
    input resetn,
    input write_1,
    input write_2,
    input write_3,
    input Lock_1,
    input Lock_2,
    input Lock_3,
    input scan_mode,
    input debug_unlocked,
    input test_mode,
    output reg [15:0] Data_out_1,
    output reg [15:0] Data_out_2,
    output reg [31:0] Data_out_3
);

reg lock_status_1;
reg lock_status_2;
reg lock_status_3;

// Lock management
always @(posedge Clk or negedge resetn)
if (~resetn) begin
    lock_status_1 <= 1'b0;
    lock_status_2 <= 1'b0;
    lock_status_3 <= 1'b0;
end
else begin
    if (Lock_1) lock_status_1 <= 1'b1;
    if (Lock_2) lock_status_2 <= 1'b1;
    if (Lock_3) lock_status_3 <= 1'b1;
end

// VULNERABILITY 1: scan_mode bypass
always @(posedge Clk or negedge resetn)
if (~resetn) begin
    Data_out_1 <= 16'h0000;
end
else if (write_1 & (~lock_status_1 | scan_mode)) begin
    Data_out_1 <= Data_in_1;
end
else begin
    Data_out_1 <= Data_out_1;
end

// VULNERABILITY 2: debug_unlocked bypass
always @(posedge Clk or negedge resetn)
if (~resetn) begin
    Data_out_2 <= 16'h0000;
end
else if (write_2 & (~lock_status_2 | debug_unlocked)) begin
    Data_out_2 <= Data_in_2;
end
else begin
    Data_out_2 <= Data_out_2;
end

// VULNERABILITY 3: Multiple bypass conditions (wider data)
always @(posedge Clk or negedge resetn)
if (~resetn) begin
    Data_out_3 <= 32'h00000000;
end
else if (write_3 & (~lock_status_3 | scan_mode | debug_unlocked | test_mode)) begin
    Data_out_3 <= Data_in_3;
end
else begin
    Data_out_3 <= Data_out_3;
end

endmodule