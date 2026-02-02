
module small_or_tree
(
input a,
input b,
input c,
input d,
input e,
output y
);

assign y = ((a | b) | (c | d));
endmodule