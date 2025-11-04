module grant_access(usr_id, correct_id, grant);
output wire grant;
input wire usr_id;
input wire correct_id;

assign grant = (usr_id == correct_id) ? 1'b1 : 1'b0;

endmodule