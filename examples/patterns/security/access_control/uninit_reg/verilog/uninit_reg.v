// examples/patterns/security/access_control/uninit_reg/verilog/uninit_reg.v
// Uninitialized register

module uninit_reg (
    input  wire        clk,
    input  wire        data_in,
    output reg         data_out
);

always @(posedge clk) begin
    data_out <= data_in;
end

endmodule