
module adff
(
input clk,
input d,
input reset_n,
output q
);

reg q1;

always @(posedge clk or negedge reset_n) begin
    if (!reset_n) q1 <= 1'b0;
    else q1 <= d;
end

assign q = q1;

endmodule