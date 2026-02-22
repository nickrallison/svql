module cdc_test(input clk_a, clk_b, data_in, output data_out);
    reg r1, r2;
    always @(posedge clk_a) r1 <= data_in;
    // Combinational path between domains
    wire logic_path = r1 ^ 1'b1;
    always @(posedge clk_b) r2 <= logic_path;
    assign data_out = r2;
endmodule