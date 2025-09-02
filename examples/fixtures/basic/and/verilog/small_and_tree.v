
module small_and_tree
(
input a,
input b,
input c,
input d,
output y
);

assign y = ((a & b) & (c & d));
endmodule