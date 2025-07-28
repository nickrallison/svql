module locked_register_example
(
input [15:0] data_in,
input clk,
input resetn,
input write,
input lock,
input scan_mode,
input debug_unlocked,
output reg [15:0] data_out
);

reg lock_status;

always @(posedge clk or negedge resetn)
    if (~resetn) // Register is reset resetn
    begin
        lock_status <= 1'b0;
    end
    else if (lock)
    begin
        lock_status <= 1'b1;
    end
    else if (~lock)
    begin
        lock_status <= lock_status;
    end
always @(posedge clk or negedge resetn)
    if (~resetn) // Register is reset resetn
    begin
        data_out <= 16'h0000;
    end
    else if (write & (~lock_status | scan_mode | debug_unlocked) ) // Register protected by lock bit input, overrides supported for scan_mode & debug_unlocked
    begin
        data_out <= data_in;
    end
    else if (~write)
    begin
        data_out <= data_out;
    end
endmodule