// Two arrays: uniform (secure), inconsistent (vulnerable).
module cwe1262_multi (
    input clk, wdata, we,
    input lock_a, lock_b,
    output reg [1:0] secure_bank [0:1],  // Uniform lock_a
    output reg [1:0] vuln_bank [0:1]     // Inconsistent: lock_a/b
);

always @(posedge clk) begin
    if (we && !lock_a) secure_bank[0] <= wdata;  // Uniform
    if (we && !lock_a) secure_bank[1] <= wdata;

    if (we && !lock_a) vuln_bank[0] <= wdata;    // lock_a
    if (we && !lock_b) vuln_bank[1] <= wdata;    // lock_b (inconsistent)
end

endmodule
