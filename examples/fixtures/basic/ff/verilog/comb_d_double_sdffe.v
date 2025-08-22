
module comb_d_double_sdffe
(
input clk,
input d,
input reset,
output wire q_w_1,
output wire q_w_2,
);

reg q1;
reg q2;

always @(posedge clk) begin
    if (reset) q1 <= 1'b0;
    else q1 <= d;
end

always @(posedge clk) begin
    if (reset) q2 <= 1'b0;
    else q2 <= d;
end

assign q_w_1 = q1;
assign q_w_2 = q2;

endmodule