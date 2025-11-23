module not_dff (
    input clk,
    input reset,
    input d,
    output reg q
);

    wire not_d;
    assign not_d = ~d;

    always @(posedge clk) begin
        if (reset)
            q <= 1'b0;
        else
            q <= not_d;
    end

endmodule