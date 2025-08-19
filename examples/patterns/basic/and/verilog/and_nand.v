// examples/patterns/basic/and/and_nand.v
module and_nand
(
    input  a,
    input  b,
    output y
);

// NAND gate (Yosys will emit a Cell::Nand)
wire nand_out;
assign nand_out = ~(a & b);

// Inverter (Cell::Not)
assign y = ~nand_out;

endmodule