// examples/fixtures/security/access_control/locked_reg/verilog/cwe1234_not_right.v
// NOT gate consistently on the right side of OR gates
// Tests both left and right input matching

module cwe1234_not_right (
    input [15:0] Data_in_1,
    input [15:0] Data_in_2,
    input [15:0] Data_in_3,
    input Clk,
    input resetn,
    input write_1,
    input write_2,
    input write_3,
    input Lock_1,
    input Lock_2,
    input Lock_3,
    input bypass_a,
    input bypass_b,
    input bypass_c,
    output reg [15:0] Data_out_1,
    output reg [15:0] Data_out_2,
    output reg [15:0] Data_out_3
);

reg lock_status_1;
reg lock_status_2;
reg lock_status_3;

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

// Pattern 1: OR(bypass, NOT(lock))
// NOT on right input of root OR
always @(posedge Clk or negedge resetn)
if (~resetn) begin
    Data_out_1 <= 16'h0000;
end
else if (write_1 & (bypass_a | ~lock_status_1)) begin
    Data_out_1 <= Data_in_1;
end
else begin
    Data_out_1 <= Data_out_1;
end

// Pattern 2: OR(bypass_a, OR(bypass_b, NOT(lock)))
// NOT on right input of nested OR
always @(posedge Clk or negedge resetn)
if (~resetn) begin
    Data_out_2 <= 16'h0000;
end
else if (write_2 & (bypass_a | (bypass_b | ~lock_status_2))) begin
    Data_out_2 <= Data_in_2;
end
else begin
    Data_out_2 <= Data_out_2;
end

// Pattern 3: OR(OR(bypass_a, bypass_b), OR(bypass_c, NOT(lock)))
// NOT on right side of right subtree
always @(posedge Clk or negedge resetn)
if (~resetn) begin
    Data_out_3 <= 16'h0000;
end
else if (write_3 & ((bypass_a | bypass_b) | (bypass_c | ~lock_status_3))) begin
    Data_out_3 <= Data_in_3;
end
else begin
    Data_out_3 <= Data_out_3;
end

endmodule