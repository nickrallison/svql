module cdc_non_violation(
    input clk,
    input data_in,
    output data_out
);
    reg q1, q2;

    always @(posedge clk) begin
        q1 <= data_in;
    end

    // Same clock domain - Safe
    always @(posedge clk) begin
        q2 <= q1;
    end

    assign data_out = q2;
endmodule