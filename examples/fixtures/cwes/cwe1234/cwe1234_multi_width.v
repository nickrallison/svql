// examples/fixtures/cwes/cwe1234/cwe1234_multi_width.v
// Multiple registers with different widths
// Tests handling of 1-bit, 8-bit, 16-bit, and 32-bit registers

module cwe1234_multi_width (
    input Data_in_1bit,
    input [7:0] Data_in_8bit,
    input [15:0] Data_in_16bit,
    input [31:0] Data_in_32bit,
    input Clk,
    input resetn,
    input write,
    input Lock,
    input debug_unlocked,
    output reg Data_out_1bit,
    output reg [7:0] Data_out_8bit,
    output reg [15:0] Data_out_16bit,
    output reg [31:0] Data_out_32bit
);

reg lock_status;

always @(posedge Clk or negedge resetn) begin
    if (~resetn) begin
        lock_status <= 1'b0;
    end else if (Lock) begin
        lock_status <= 1'b1;
    end
end

// 1-bit register
always @(posedge Clk or negedge resetn) begin
    if (~resetn) begin
        Data_out_1bit <= 1'b0;
    end else if (write & (~lock_status | debug_unlocked)) begin
        Data_out_1bit <= Data_in_1bit;
    end
end

// 8-bit register
always @(posedge Clk or negedge resetn) begin
    if (~resetn) begin
        Data_out_8bit <= 8'h00;
    end else if (write & (~lock_status | debug_unlocked)) begin
        Data_out_8bit <= Data_in_8bit;
    end
end

// 16-bit register
always @(posedge Clk or negedge resetn) begin
    if (~resetn) begin
        Data_out_16bit <= 16'h0000;
    end else if (write & (~lock_status | debug_unlocked)) begin
        Data_out_16bit <= Data_in_16bit;
    end
end

// 32-bit register
always @(posedge Clk or negedge resetn) begin
    if (~resetn) begin
        Data_out_32bit <= 32'h00000000;
    end else if (write & (~lock_status | debug_unlocked)) begin
        Data_out_32bit <= Data_in_32bit;
    end
end

endmodule