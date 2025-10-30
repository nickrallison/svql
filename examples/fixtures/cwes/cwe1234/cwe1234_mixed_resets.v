// examples/fixtures/cwes/cwe1234/cwe1234_mixed_resets.v
// Mixes async and sync reset DFFs in same design
// Tests handling of heterogeneous register types

module cwe1234_mixed_resets (
    input [15:0] Data_in_1,
    input [15:0] Data_in_2,
    input Clk,
    input resetn,
    input write_1,
    input write_2,
    input Lock_1,
    input Lock_2,
    input debug_unlocked,
    output reg [15:0] Data_out_1,
    output reg [15:0] Data_out_2
);

reg lock_status_1;
reg lock_status_2;

// Async reset for lock 1
always @(posedge Clk or negedge resetn) begin
    if (~resetn) begin
        lock_status_1 <= 1'b0;
    end else if (Lock_1) begin
        lock_status_1 <= 1'b1;
    end
end

// Sync reset for lock 2
always @(posedge Clk) begin
    if (~resetn) begin
        lock_status_2 <= 1'b0;
    end else if (Lock_2) begin
        lock_status_2 <= 1'b1;
    end
end

// VULNERABILITY 1: Async reset data register
always @(posedge Clk or negedge resetn) begin
    if (~resetn) begin
        Data_out_1 <= 16'h0000;
    end else if (write_1 & (~lock_status_1 | debug_unlocked)) begin
        Data_out_1 <= Data_in_1;
    end
end

// VULNERABILITY 2: Sync reset data register
always @(posedge Clk) begin
    if (~resetn) begin
        Data_out_2 <= 16'h0000;
    end else if (write_2 & (~lock_status_2 | debug_unlocked)) begin
        Data_out_2 <= Data_in_2;
    end
end

endmodule