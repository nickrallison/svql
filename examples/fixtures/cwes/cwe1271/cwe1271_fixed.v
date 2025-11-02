// examples/fixtures/cwes/cwe1271/cwe1271_fixed.v

module cwe1271_fixed (
    input  wire        clk,
    input  wire        data_in,
    input  wire        resetn,
    output reg         data_out
);

always @(posedge clk or negedge resetn) begin
    if (~resetn) begin
        data_out <= 1'b0;
    end else begin
        data_out <= data_in;
    end
end

endmodule