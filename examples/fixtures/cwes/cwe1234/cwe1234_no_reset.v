
module cwe1234_no_reset (
    input [15:0] Data_in,
    input Clk,
    input write,
    input Lock,
    input debug_unlocked,
    output reg [15:0] Data_out
);

reg lock_status;

always @(posedge Clk) begin
    if (Lock) begin
        lock_status <= 1'b1;
    end
end

always @(posedge Clk) begin
    if (write & (~lock_status | debug_unlocked)) begin
        Data_out <= Data_in;
    end
end

endmodule