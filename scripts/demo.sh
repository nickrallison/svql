#!/bin/sh

mkdir generated
yosys -p "read_rtlil examples/patterns/security/access_control/locked_reg/rtlil/sync_en.il" \
      -p "read_rtlil examples/patterns/security/access_control/locked_reg/rtlil/sync_mux.il" \
      -p "read_rtlil examples/patterns/security/access_control/locked_reg/rtlil/async_en.il" \
      -p "read_rtlil examples/patterns/security/access_control/locked_reg/rtlil/async_mux.il" \
      -p "read_verilog examples/patterns/security/access_control/locked_reg/verilog/many_locked_regs.v" \
      -p "hierarchy -top many_locked_regs" \
      -p "flatten" \
      -p "write_rtlil generated/many_locked_regs.il" \
      -p "show -format dot -prefix generated/many_locked_regs"

dot -Tpdf generated/many_locked_regs.dot -o generated/many_locked_regs.pdf