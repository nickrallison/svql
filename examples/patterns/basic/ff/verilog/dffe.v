module dffe
(
    input clk,
    input d,
    input en,
    output q
);

reg q1;

always @(posedge clk) begin
    if (en) q1 <= d;
end

assign q = q1;

endmodule