// examples/patterns/security/access_control/locked_reg/verilog/async_en.v
// Async reset flip-flop with explicit enable signal
// Synthesizes to $adffe cell

module async_en (
    input  wire        clk,
    input  wire        data_in,
    input  wire        resetn,
    input  wire        write_en,
    output reg         data_out
);

always @(posedge clk or negedge resetn) begin
    if (~resetn) begin
        data_out <= 1'h0;
    end else if (write_en) begin
        data_out <= data_in;
    end
    // Implicit: else data_out <= data_out (hold current value)
end

endmodule