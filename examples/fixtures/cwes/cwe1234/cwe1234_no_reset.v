// examples/fixtures/cwes/cwe1234/cwe1234_no_reset.v
// DFFs without reset (uncommon but valid)
// Tests handling of minimal DFF structures

module cwe1234_no_reset (
    input [15:0] Data_in,
    input Clk,
    input write,
    input Lock,
    input debug_unlocked,
    output reg [15:0] Data_out
);

reg lock_status;

// No reset - lock starts undefined
always @(posedge Clk) begin
    if (Lock) begin
        lock_status <= 1'b1;
    end
end

// VULNERABILITY: No reset, simple bypass
always @(posedge Clk) begin
    if (write & (~lock_status | debug_unlocked)) begin
        Data_out <= Data_in;
    end
end

endmodule