module sdffe (
    input clk,
    input reset,
    input en,
    input d,
    output reg q
);
    always @(posedge clk) begin
        if (reset) begin
            q <= 1'b0;
        end else if (en) begin
            q <= d;
        end
    end
endmodule