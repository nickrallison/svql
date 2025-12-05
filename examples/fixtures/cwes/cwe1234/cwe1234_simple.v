
module cwe1234_simple (
    input [15:0] Data_in,
    input Clk,
    input resetn,
    input write,
    input Lock,
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

always @(posedge Clk or negedge resetn)
if (~resetn) begin
    Data_out <= 16'h0000;
end
else if (write & (~lock_status | debug_unlocked)) begin
    Data_out <= Data_in;
end
else begin
    Data_out <= Data_out;
end

endmodule