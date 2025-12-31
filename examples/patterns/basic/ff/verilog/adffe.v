module adffe (
    input clk,
    input reset_n,
    input en,
    input d,
    output reg q
);
    always @(posedge clk or negedge reset_n) begin
        if (!reset_n) begin
            q <= 1'b0;
        end else if (en) begin
            q <= d;
        end
    end
endmodule