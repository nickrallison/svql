
module small_and_seq
(
input a,
input b,
input c,
input d,
output y
);

assign y = ((a & b) & c);
endmodule