module grant_access(usr_id, correct_id, grant_access);
output wire grant_access;
input wire [2:0] usr_id;
input wire [2:0] correct_id;

grant_access = (usr_id == correct_id) ? 1'b1 : 1'b0;

endmodule