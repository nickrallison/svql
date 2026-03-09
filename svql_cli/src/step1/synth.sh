#!/bin/sh

yosys -p "read_verilog svql_cli/src/step1/adc_test.v" -p "hierarchy -check -top adc_test" -p proc -p fsm -p opt -p "write_rtlil svql_cli/src/step1/adc_test.il"