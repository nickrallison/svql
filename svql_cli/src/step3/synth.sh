#!/bin/sh

yosys -p "read_verilog svql_cli/src/step3/mixed_fa_test.v" -p "hierarchy -check -top mixed_fa_test" -p flatten -p opt_clean -p "write_rtlil svql_cli/src/step3/mixed_fa_test.il"