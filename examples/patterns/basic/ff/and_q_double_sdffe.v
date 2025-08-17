
module and_q_double_sdffe
(
input clk,
input d1,
input d2,
input reset,
output wire q,
);

reg q1;
reg q2;

always @(posedge clk) begin
    if (reset) q1 <= 1'b0;
    else q1 <= d1;
end

always @(posedge clk) begin
    if (reset) q2 <= 1'b0;
    else q2 <= d2;
end

assign q = q1 & q2;
endmodule