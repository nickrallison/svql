module cdc_multi_bit_violation(
    input clk_a,
    input clk_b,
    input [1:0] data_in,
    output [1:0] data_out
);
    reg [1:0] q_src;
    reg [1:0] q_dst;

    // Source register on clk_a
    always @(posedge clk_a) begin
        q_src <= data_in;
    end

    // Destination bit 0: Same clock as source (Safe)
    always @(posedge clk_a) begin
        q_dst[0] <= q_src[0];
    end

    // Destination bit 1: Different clock (Violation)
    always @(posedge clk_b) begin
        q_dst[1] <= q_src[1];
    end

    assign data_out = q_dst;
endmodule