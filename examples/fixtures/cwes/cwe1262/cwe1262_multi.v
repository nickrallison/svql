// Two arrays: uniform (secure), inconsistent (vulnerable).
module cwe1262_multi (
    input clk, 
    input [1:0] wdata, 
    input we,
    input lock_a, lock_b,
    output reg [1:0] secure_bank0,
    output reg [1:0] secure_bank1,
    output reg [1:0] vuln_bank0,
    output reg [1:0] vuln_bank1
);

always @(posedge clk) begin
    if (we && !lock_a) begin
        secure_bank0 <= wdata;
        secure_bank1 <= wdata;
    end

    if (we && !lock_a) vuln_bank0 <= wdata; 
    if (we && !lock_b) vuln_bank1 <= wdata;
end

endmodule