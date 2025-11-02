// All array entries protected by same lock (secure).
module cwe1262_uniform (
    input clk,
    input [31:0] wdata,
    input we,
    input lock,
    output reg [31:0] reg_bank0,
    output reg [31:0] reg_bank1,
    output reg [31:0] reg_bank2,
    output reg [31:0] reg_bank3
);

always @(posedge clk) begin
    if (we && !lock) begin
        reg_bank0 <= wdata;
        reg_bank1 <= wdata;
        reg_bank2 <= wdata;
        reg_bank3 <= wdata;
    end
end

endmodule