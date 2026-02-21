#!/bin/sh

yosys -p "read_verilog svql_cli/src/step2/full_adder_from_half_adders.v" -p "hierarchy -check -top full_adder_from_half_adders" -p flatten -p "write_rtlil svql_cli/src/step2/full_adder_from_half_adders.il"