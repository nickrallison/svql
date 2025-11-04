module cwe1280_basic (
    input [7:0] usr_id,
    input [7:0] correct_id,
    input clk,
    output reg [7:0] protected_data
);
    wire grant = (usr_id == correct_id);  // Weak ID check
    always @(posedge clk) begin
        if (grant) protected_data <= some_data;  // Bypass vuln
    end
endmodule