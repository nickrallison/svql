// examples/patterns/security/access_control/locked_reg/verilog/sync_mux.v
// Sync reset flip-flop with mux-based enable (alternative to explicit enable)
// Synthesizes to $dff cell + $mux cells for enable and reset logic

module sync_mux (
    input  wire        clk,
    input  wire        data_in,
    input  wire        resetn,
    input  wire        write_en,
    output reg         data_out
);

always @(posedge clk) begin
    if (~resetn) begin
        data_out <= 1'h0;
    end else begin
        // Mux-style enable: write_en ? data_in : data_out
        // Yosys will synthesize this as separate muxes (one for reset, one for enable)
        data_out <= write_en ? data_in : data_out;
    end
end

endmodule