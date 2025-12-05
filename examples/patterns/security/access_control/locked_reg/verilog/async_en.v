
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
end

endmodule