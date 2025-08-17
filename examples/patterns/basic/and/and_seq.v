
module and_seq
(
input a,
input b,
input c,
input d,
input e,
input f,
input g,
input h,
output y
);

assign y = (((((((a & b) & c) & d) & e) & f) & g) & h);
endmodule