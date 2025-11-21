module dff_loop_toggle (
    input clk,
    input reset,
    output reg q
);
    always @(posedge clk) begin
        if (reset)
            q <= 1'b0;
        else
            q <= ~q;
    end
endmodule