
module cwe1234_enabled (
    input [15:0] Data_in,
    input Clk,
    input resetn,
    input write,
    input Lock,
    input debug_unlocked,
    output reg [15:0] Data_out
);

reg lock_status;
wire lock_enable;
wire data_enable;

assign lock_enable = Lock;

always @(posedge Clk or negedge resetn) begin
    if (~resetn) begin
        lock_status <= 1'b0;
    end else if (lock_enable) begin
        lock_status <= 1'b1;
    end
end

assign data_enable = write & (~lock_status | debug_unlocked);

always @(posedge Clk or negedge resetn) begin
    if (~resetn) begin
        Data_out <= 16'h0000;
    end else if (data_enable) begin
        Data_out <= Data_in;
    end
end

endmodule