
module sdffe
(
input clk,
input d,
input reset,
output q
);

reg q1;

always @(posedge clk) begin
    if (reset) q1 <= 1'b0;
    else q1 <= d;
end

assign q = q1;

endmodule