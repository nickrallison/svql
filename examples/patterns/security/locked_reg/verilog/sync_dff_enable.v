// Pattern: Sync reset DFF with explicit enable signal
module sync_dff_enable (
    input  clk,
    input  d,
    input  resetn,
    input  enable,
    output q
);

reg q_reg;

always @(posedge clk) begin
    if (!resetn)
        q_reg <= 1'b0;
    else if (enable)
        q_reg <= d;
end

assign q = q_reg;

endmodule