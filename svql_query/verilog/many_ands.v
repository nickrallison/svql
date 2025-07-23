

module many_ands
(
input a,
input b,
input c,
// input d,
// input e,
// input f,
output y
);


// assign y = ((a & b) & c) & ((d & e) & f);
assign y = (((a & b) & c) & d);
endmodule