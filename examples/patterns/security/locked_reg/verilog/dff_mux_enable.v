// Pattern: DFF with MUX-based enable (what Yosys typically generates)
// The MUX selects between new data (when enabled) and feedback (when disabled)
module dff_mux_enable (
    input  clk,
    input  d,        // New data input
    input  resetn,
    input  enable,   // Enable/write signal
    output q
);

reg q_reg;
wire mux_out;

// MUX: when enable is high, pass new data; otherwise feedback current value
assign mux_out = enable ? d : q_reg;

always @(posedge clk or negedge resetn) begin
    if (!resetn)
        q_reg <= 1'b0;
    else
        q_reg <= mux_out;
end

assign q = q_reg;

endmodule