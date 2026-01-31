
module small_and_tree
(
input a,
input b,
input c,
input d,
input e,
output y
);

assign y = ((a & b) & (c & d));
endmodule