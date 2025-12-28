
# Usage: ./verific_to_json.sh <verilog_filepath> <top_module_name>
# E.g. ./verific_to_json.sh examples/fixtures/larger_designs/verilog/hackatdac18/soc_peripherals_netlist.v soc_peripherals

export FILEPATH=$1;
export TOP=$2;

yosys -p "read_json $FILEPATH" \
      -p "hierarchy -check -top $TOP" \
      -p "proc" \
      -p "memory" \
      -p "opt_clean" \
      -p "flatten" \
      -p "opt_clean" \
      -p "write_json $FILEPATH"