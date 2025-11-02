// Dummy protected DFF entry for synthesis.
module prot_dff_entry (
    input clk,
    input wdata,
    input we,
    input lock,
    output reg q
);

always @(posedge clk) begin
    if (we && !lock) q <= wdata;
end

endmodule
