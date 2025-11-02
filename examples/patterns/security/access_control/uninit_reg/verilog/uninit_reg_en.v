// examples/patterns/security/access_control/uninit_reg/verilog/uninit_reg_en.v
// Uninitialized register

module uninit_reg_en (
    input  wire        clk,
    input  wire        data_in,
    input  wire        write_en,
    output reg         data_out
);

always @(posedge clk) begin
    if (write_en) begin
        data_out <= data_in;
    end
end

endmodule