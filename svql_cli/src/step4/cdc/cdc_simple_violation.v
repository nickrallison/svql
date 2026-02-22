module cdc_simple_violation(
    input clk_a,
    input clk_b,
    input data_in,
    output data_out
);
    reg q_a;
    reg q_b;

    // Domain A
    always @(posedge clk_a) begin
        q_a <= data_in;
    end

    // Combinational path (Logic Cone)
    wire combined = q_a & 1'b1;

    // Domain B - Violation: q_a is not synchronized to clk_b
    always @(posedge clk_b) begin
        q_b <= combined;
    end

    assign data_out = q_b;
endmodule