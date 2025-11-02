// Array with varying locks (reg[0-1] use lock1, reg[2-3] use lock2).
module cwe1262_inconsistent (
    input clk,
    input [31:0] wdata,
    input we,
    input lock1, lock2,  // Different locks
    output reg [31:0] reg_bank [0:3]
);

always @(posedge clk) begin
    if (we && !lock1) reg_bank[0] <= wdata;  // Protected by lock1
    if (we && !lock1) reg_bank[1] <= wdata;  // Protected by lock1
    if (we && !lock2) reg_bank[2] <= wdata;  // Protected by lock2 (inconsistent)
    if (we && !lock2) reg_bank[3] <= wdata;  // Protected by lock2
end

endmodule
