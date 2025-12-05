
module cwe1234_mixed_resets (
    input [15:0] Data_in_1,
    input [15:0] Data_in_2,
    input Clk,
    input resetn,
    input write_1,
    input write_2,
    input Lock_1,
    input Lock_2,
    input debug_unlocked,
    output reg [15:0] Data_out_1,
    output reg [15:0] Data_out_2
);

reg lock_status_1;
reg lock_status_2;

always @(posedge Clk or negedge resetn) begin
    if (~resetn) begin
        lock_status_1 <= 1'b0;
    end else if (Lock_1) begin
        lock_status_1 <= 1'b1;
    end
end

always @(posedge Clk) begin
    if (~resetn) begin
        lock_status_2 <= 1'b0;
    end else if (Lock_2) begin
        lock_status_2 <= 1'b1;
    end
end

always @(posedge Clk or negedge resetn) begin
    if (~resetn) begin
        Data_out_1 <= 16'h0000;
    end else if (write_1 & (~lock_status_1 | debug_unlocked)) begin
        Data_out_1 <= Data_in_1;
    end
end

always @(posedge Clk) begin
    if (~resetn) begin
        Data_out_2 <= 16'h0000;
    end else if (write_2 & (~lock_status_2 | debug_unlocked)) begin
        Data_out_2 <= Data_in_2;
    end
end

endmodule