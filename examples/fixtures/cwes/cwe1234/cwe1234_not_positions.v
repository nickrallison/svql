// examples/fixtures/security/access_control/locked_reg/verilog/cwe1234_not_positions.v
// Multiple registers with NOT gate at different depths in OR tree
// Tests recursive tree traversal finds NOT regardless of position

module cwe1234_not_positions (
    input [15:0] Data_in_1,
    input [15:0] Data_in_2,
    input [15:0] Data_in_3,
    input [15:0] Data_in_4,
    input Clk,
    input resetn,
    input write_1,
    input write_2,
    input write_3,
    input write_4,
    input Lock_1,
    input Lock_2,
    input Lock_3,
    input Lock_4,
    input scan_mode,
    input debug_unlocked,
    input test_mode,
    output reg [15:0] Data_out_1,
    output reg [15:0] Data_out_2,
    output reg [15:0] Data_out_3,
    output reg [15:0] Data_out_4
);

reg lock_status_1;
reg lock_status_2;
reg lock_status_3;
reg lock_status_4;

always @(posedge Clk or negedge resetn)
if (~resetn) begin
    lock_status_1 <= 1'b0;
    lock_status_2 <= 1'b0;
    lock_status_3 <= 1'b0;
    lock_status_4 <= 1'b0;
end
else begin
    if (Lock_1) lock_status_1 <= 1'b1;
    if (Lock_2) lock_status_2 <= 1'b1;
    if (Lock_3) lock_status_3 <= 1'b1;
    if (Lock_4) lock_status_4 <= 1'b1;
end

// PATTERN 1: NOT at depth 1, leftmost position
// Synthesized as: write & (~lock | scan | debug)
//                         ^^^^^
//                       NOT is first input
always @(posedge Clk or negedge resetn)
if (~resetn) begin
    Data_out_1 <= 16'h0000;
end
else if (write_1 & (~lock_status_1 | scan_mode | debug_unlocked)) begin
    Data_out_1 <= Data_in_1;
end
else begin
    Data_out_1 <= Data_out_1;
end

// PATTERN 2: NOT at depth 1, middle position
// Synthesized as: write & (scan | ~lock | debug)
//                               ^^^^^
//                            NOT is middle input
always @(posedge Clk or negedge resetn)
if (~resetn) begin
    Data_out_2 <= 16'h0000;
end
else if (write_2 & (scan_mode | ~lock_status_2 | debug_unlocked)) begin
    Data_out_2 <= Data_in_2;
end
else begin
    Data_out_2 <= Data_out_2;
end

// PATTERN 3: NOT at depth 1, rightmost position
// Synthesized as: write & (scan | debug | ~lock)
//                                       ^^^^^
//                                    NOT is last input
always @(posedge Clk or negedge resetn)
if (~resetn) begin
    Data_out_3 <= 16'h0000;
end
else if (write_3 & (scan_mode | debug_unlocked | ~lock_status_3)) begin
    Data_out_3 <= Data_in_3;
end
else begin
    Data_out_3 <= Data_out_3;
end

// PATTERN 4: NOT at depth 2, nested on left side
// Synthesized as: write & ((~lock | scan) | (debug | test))
//                           ^^^^^
//                        NOT is nested left
always @(posedge Clk or negedge resetn)
if (~resetn) begin
    Data_out_4 <= 16'h0000;
end
else if (write_4 & ((~lock_status_4 | scan_mode) | (debug_unlocked | test_mode))) begin
    Data_out_4 <= Data_in_4;
end
else begin
    Data_out_4 <= Data_out_4;
end

endmodule