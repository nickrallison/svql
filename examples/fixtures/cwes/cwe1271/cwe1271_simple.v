// examples/fixtures/cwes/cwe1271/cwe1271_simple.v
// Basic uninitialized DFF (no reset, no enable)
// Matches UninitReg pattern: always @(posedge clk) data_out <= data_in;

module cwe1271_simple (
    input  wire        clk,
    input  wire        data_in,
    output reg         data_out
);

always @(posedge clk) begin
    data_out <= data_in; 
end

endmodule