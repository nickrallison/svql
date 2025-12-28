


## 1. Decompress Verilog netlist files
gzip -d -k examples/fixtures/larger_designs/verilog/hackatdac18/cv32e40p_fp_wrapper_netlist.v.gz
gzip -d -k examples/fixtures/larger_designs/verilog/hackatdac18/soc_interconnect_wrap_netlist.v.gz
gzip -d -k examples/fixtures/larger_designs/verilog/hackatdac18/soc_peripherals_netlist.v.gz
gzip -d -k examples/fixtures/larger_designs/verilog/hackatdac18/udma_core_netlist.v.gz

gzip -d -k examples/fixtures/larger_designs/verilog/hackatdac21/openpiton_tile.v.gz

gzip -f -k examples/fixtures/larger_designs/verilog/hummingbirdv2/e203_soc_netlist.v.gz



## 2. Convert Verilog netlist files to JSON using Yosys

mkdir -p examples/fixtures/larger_designs/json/hackatdac18/
mkdir -p examples/fixtures/larger_designs/json/hackatdac21/
mkdir -p examples/fixtures/larger_designs/json/hummingbirdv2/

sh scripts/verific_to_json.sh examples/fixtures/larger_designs/verilog/hackatdac18/cv32e40p_fp_wrapper_netlist.v cv32e40p_fp_wrapper
sh scripts/verific_to_json.sh examples/fixtures/larger_designs/verilog/hackatdac18/soc_interconnect_wrap_netlist.v soc_interconnect_wrap
sh scripts/verilog_to_json.sh examples/fixtures/larger_designs/verilog/hackatdac18/soc_peripherals_netlist.v soc_peripherals
sh scripts/verific_to_json.sh examples/fixtures/larger_designs/verilog/hackatdac18/udma_core_netlist.v udma_core

sh scripts/verilog_to_json.sh examples/fixtures/larger_designs/verilog/hackatdac21/openpiton_tile.v tile

sh scripts/verific_to_json.sh examples/fixtures/larger_designs/verilog/hummingbirdv2/e203_soc_netlist.v e203_soc_top