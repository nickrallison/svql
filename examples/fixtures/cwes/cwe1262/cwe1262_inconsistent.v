// Array with varying locks (entries 0-1 use lock1, 2-3 use lock2).
module cwe1262_inconsistent (
    input clk,
    input [31:0] wdata,
    input we,
    input lock1, lock2,
    output reg [31:0] reg_bank0,
    output reg [31:0] reg_bank1,
    output reg [31:0] reg_bank2,
    output reg [31:0] reg_bank3
);

always @(posedge clk) begin
    if (we && !lock1) reg_bank0 <= wdata;
    if (we && !lock1) reg_bank1 <= wdata;
    if (we && !lock2) reg_bank2 <= wdata;
    if (we && !lock2) reg_bank3 <= wdata; 
end

endmodule