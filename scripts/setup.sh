
## 1. Decompress Verilog netlist files
gzip -d -k examples/fixtures/larger_designs/verilog/hackatdac18/cv32e40p_fp_wrapper_netlist.v.gz
gzip -d -k examples/fixtures/larger_designs/verilog/hackatdac18/soc_interconnect_wrap_netlist.v.gz
gzip -d -k examples/fixtures/larger_designs/verilog/hackatdac18/soc_peripherals_netlist.v.gz
gzip -d -k examples/fixtures/larger_designs/verilog/hackatdac18/udma_core_netlist.v.gz
gzip -d -k examples/fixtures/larger_designs/verilog/hackatdac21/openpiton_tile.v.gz

## 2. Convert Verilog netlist files to JSON using Yosys
sh scripts/verific_to_json.sh examples/fixtures/larger_designs/verilog/hackatdac18/cv32e40p_fp_wrapper_netlist.v cv32e40p_fp_wrapper
sh scripts/verific_to_json.sh examples/fixtures/larger_designs/verilog/hackatdac18/soc_interconnect_wrap_netlist.v soc_interconnect_wrap
sh scripts/verilog_to_json.sh examples/fixtures/larger_designs/verilog/hackatdac18/soc_peripherals_netlist.v soc_peripherals
sh scripts/verific_to_json.sh examples/fixtures/larger_designs/verilog/hackatdac18/udma_core_netlist.v udma_core
sh scripts/verilog_to_json.sh examples/fixtures/larger_designs/verilog/hackatdac21/openpiton_tile.v tile