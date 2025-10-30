// Pattern: Async reset DFF with explicit enable signal
// Matches DFFs where enable controls when data is latched
module async_dff_enable (
    input  clk,
    input  d,
    input  resetn,
    input  enable,  // This is what we'll connect to unlock logic
    output q
);

reg q_reg;

always @(posedge clk or negedge resetn) begin
    if (!resetn)
        q_reg <= 1'b0;
    else if (enable)
        q_reg <= d;
    // else q_reg stays same (implicit latch behavior)
end

assign q = q_reg;

endmodule