
module seq_double_sdffe
(
input clk,
input d,
input reset,
output wire q
);

reg q1;
reg q2;

always @(posedge clk) begin
    if (reset) q1 <= 1'b0;
    else q1 <= d;
end

always @(posedge clk) begin
    if (reset) q2 <= 1'b0;
    else q2 <= q1;
end

assign q = q2;

endmodule