
module sync_mux (
    input  wire        clk,
    input  wire        data_in,
    input  wire        resetn,
    input  wire        write_en,
    output reg         data_out
);

always @(posedge clk) begin
    if (~resetn) begin
        data_out <= 1'h0;
    end else begin
        data_out <= write_en ? data_in : data_out;
    end
end

endmodule