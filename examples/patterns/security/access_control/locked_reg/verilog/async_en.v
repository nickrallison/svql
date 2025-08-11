module async_en
(
input [15:0] data_in,
input clk,
input resetn,
input write_en,
output reg [15:0] data_out
);

always @(posedge clk or negedge resetn)
    if (~resetn) // Register is reset resetn
    begin
        data_out <= 16'h0000;
    end
    else if (write_en)
    begin
        data_out <= data_in;
    end
    else
    begin
        data_out <= data_out;
    end
endmodule