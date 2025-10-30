// examples/patterns/security/access_control/locked_reg/verilog/async_mux.v
// Async reset flip-flop with mux-based enable (alternative to explicit enable)
// Synthesizes to $adff cell + $mux cell for enable logic

module async_mux (
    input  wire        clk,
    input  wire        data_in,
    input  wire        resetn,
    input  wire        write_en,
    output reg         data_out
);

always @(posedge clk or negedge resetn) begin
    if (~resetn) begin
        data_out <= 1'h0;
    end else begin
        // Mux-style enable: write_en ? data_in : data_out
        // Yosys will synthesize this as a separate mux feeding the DFF
        data_out <= write_en ? data_in : data_out;
    end
end

endmodule