module primitive_ha_test(input [3:0] a, b, output [3:0] sum, output [3:0] diff);
    assign sum = a + b;
    assign diff = a - b;
endmodule