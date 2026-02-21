module adc_test(input [3:0] a, b, output [3:0] sum, output [3:0] diff);
    // Yosys will map these to $add and $sub cells (AdcGate)
    assign sum = a + b;
    assign diff = a - b;
endmodule