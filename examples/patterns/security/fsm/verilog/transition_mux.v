// Dummy transition mux.
module transition_mux (
    input [3:0] state,
    input cond_a, cond_b,
    output reg [3:0] next_state
);

always @* begin
    case (state)
        4'b0001: next_state = cond_a ? 4'b0010 : 4'b0001;
        4'b0010: next_state = cond_b ? 4'b0100 : 4'b0001;
        4'b0100: next_state = 4'b1000;
        4'b1000: next_state = 4'b0001;
        default: next_state = 4'b0001;
    endcase
end

endmodule
