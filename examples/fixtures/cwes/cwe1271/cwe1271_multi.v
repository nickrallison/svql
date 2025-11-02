// examples/fixtures/cwes/cwe1271/cwe1271_multi.v

module cwe1271_multi (
    input  wire        clk,
    input  wire        data_in_1,
    input  wire        data_in_2,
    input  wire        data_in_3,
    input  wire        write_en_3,
    output reg         data_out_1,
    output reg         data_out_2,
    output reg         data_out_3
);

always @(posedge clk) begin
    data_out_1 <= data_in_1;
end

always @(posedge clk) begin
    data_out_2 <= data_in_2;
end

always @(posedge clk) begin
    if (write_en_3) begin
        data_out_3 <= data_in_3;
    end
end

endmodule