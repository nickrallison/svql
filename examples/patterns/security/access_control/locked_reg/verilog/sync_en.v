// examples/patterns/security/access_control/locked_reg/verilog/sync_en.v
// Sync reset flip-flop with explicit enable signal
// Synthesizes to $sdffe cell

module sync_en (
    input  wire        clk,
    input  wire        data_in,
    input  wire        resetn,
    input  wire        write_en,
    output reg         data_out
);

always @(posedge clk) begin
    if (~resetn) begin
        data_out <= 1'h0;
    end else if (write_en) begin
        data_out <= data_in;
    end
    // Implicit: else data_out <= data_out (hold current value)
end

endmodule