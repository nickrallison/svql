#!/bin/sh

yosys -p "read_verilog svql_cli/src/step3/mixed_ha_test.v" -p "hierarchy -check -top mixed_ha_test" -p flatten -p opt_clean -p "write_rtlil svql_cli/src/step3/mixed_ha_test.il"