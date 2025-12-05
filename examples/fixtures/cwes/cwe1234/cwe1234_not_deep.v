
module cwe1234_not_deep (
    input [15:0] Data_in_shallow,
    input [15:0] Data_in_mid,
    input [15:0] Data_in_deep,
    input Clk,
    input resetn,
    input write_shallow,
    input write_mid,
    input write_deep,
    input Lock_shallow,
    input Lock_mid,
    input Lock_deep,
    input bypass_1,
    input bypass_2,
    input bypass_3,
    input bypass_4,
    input bypass_5,
    output reg [15:0] Data_out_shallow,
    output reg [15:0] Data_out_mid,
    output reg [15:0] Data_out_deep
);

reg lock_status_shallow;
reg lock_status_mid;
reg lock_status_deep;

always @(posedge Clk or negedge resetn)
if (~resetn) begin
    lock_status_shallow <= 1'b0;
    lock_status_mid <= 1'b0;
    lock_status_deep <= 1'b0;
end
else begin
    if (Lock_shallow) lock_status_shallow <= 1'b1;
    if (Lock_mid) lock_status_mid <= 1'b1;
    if (Lock_deep) lock_status_deep <= 1'b1;
end

always @(posedge Clk or negedge resetn)
if (~resetn) begin
    Data_out_shallow <= 16'h0000;
end
else if (write_shallow & (~lock_status_shallow | bypass_1)) begin
    Data_out_shallow <= Data_in_shallow;
end
else begin
    Data_out_shallow <= Data_out_shallow;
end

always @(posedge Clk or negedge resetn)
if (~resetn) begin
    Data_out_mid <= 16'h0000;
end
else if (write_mid & ((~lock_status_mid | bypass_1) | bypass_2)) begin
    Data_out_mid <= Data_in_mid;
end
else begin
    Data_out_mid <= Data_out_mid;
end

always @(posedge Clk or negedge resetn)
if (~resetn) begin
    Data_out_deep <= 16'h0000;
end
else if (write_deep & (((~lock_status_deep | bypass_1) | bypass_2) | (bypass_3 | (bypass_4 | bypass_5)))) begin
    Data_out_deep <= Data_in_deep;
end
else begin
    Data_out_deep <= Data_out_deep;
end

endmodule