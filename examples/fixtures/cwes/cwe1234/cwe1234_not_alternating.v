
module cwe1234_not_alternating (
    input [15:0] Data_in_1,
    input [15:0] Data_in_2,
    input Clk,
    input resetn,
    input write_1,
    input write_2,
    input Lock_1,
    input Lock_2,
    input bypass_a,
    input bypass_b,
    input bypass_c,
    output reg [15:0] Data_out_1,
    output reg [15:0] Data_out_2
);

reg lock_status_1;
reg lock_status_2;

always @(posedge Clk or negedge resetn)
if (~resetn) begin
    lock_status_1 <= 1'b0;
    lock_status_2 <= 1'b0;
end
else begin
    if (Lock_1) lock_status_1 <= 1'b1;
    if (Lock_2) lock_status_2 <= 1'b1;
end

always @(posedge Clk or negedge resetn)
if (~resetn) begin
    Data_out_1 <= 16'h0000;
end
else if (write_1 & (~lock_status_1 | (bypass_a | bypass_b))) begin
    Data_out_1 <= Data_in_1;
end
else begin
    Data_out_1 <= Data_out_1;
end

always @(posedge Clk or negedge resetn)
if (~resetn) begin
    Data_out_2 <= 16'h0000;
end
else if (((bypass_a | bypass_b) | ~lock_status_2) & write_2) begin
    Data_out_2 <= Data_in_2;
end
else begin
    Data_out_2 <= Data_out_2;
end

endmodule