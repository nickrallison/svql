// examples/fixtures/cwes/cwe1271/cwe1271_en.v

module cwe1271_en (
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