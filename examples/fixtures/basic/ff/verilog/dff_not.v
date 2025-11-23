module dff_not (
    input clk,
    input reset,
    input d,
    output reg q,
    output wire not_q
);

    always @(posedge clk) begin
        if (reset)
            q <= 1'b0;
        else
            q <= d;
    end

    assign not_q = ~q;
endmodule