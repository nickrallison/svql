module foo_bar(data_out, usr_id, data_in, clk, rst_n);
output reg [7:0] data_out;
input wire [2:0] usr_id;
input wire [7:0] data_in;
input wire clk, rst_n;
reg grant_access;
always @ (posedge clk or negedge rst_n)
begin
if (!rst_n)
grant_access <= 0;
data_out = 0;
else
data_out = (grant_access) ? data_in : data_out;
assign grant_access = (usr_id == 3'h4) ? 1'b1 : 1'b0;
end
endmodule 