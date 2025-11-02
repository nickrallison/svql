// All array entries protected by same lock (secure).
module cwe1262_uniform (
    input clk,
    input [31:0] wdata,
    input we,
    input lock,  // Single lock for all
    output reg [31:0] reg_bank [0:3]
);

always @(posedge clk) begin
    if (we && !lock) begin  // Uniform protection
        reg_bank[0] <= wdata;
        reg_bank[1] <= wdata;
        reg_bank[2] <= wdata;
        reg_bank[3] <= wdata;
    end
end

endmodule
