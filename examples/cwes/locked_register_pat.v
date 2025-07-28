module locked_register_example
(
input [15:0] data_in,
input clk,
input resetn,
input write,
input not_lock_status,
input lock_override,
output reg [15:0] data_out
);

always @(posedge clk or negedge resetn)
    if (~resetn) // Register is reset resetn
    begin
        data_out <= 16'h0000;
    end
    else if (write & (not_lock_status | lock_override))
    begin
        data_out <= data_in;
    end
    else if (~write)
    begin
        data_out <= data_out;
    end
endmodule