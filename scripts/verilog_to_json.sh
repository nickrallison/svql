
# Usage: ./verilog_to_json.sh <verilog_filepath> <top_module_name>
# E.g. ./verilog_to_json.sh examples/fixtures/larger_designs/verilog/hackatdac18/soc_peripherals_netlist.v soc_peripherals

export FILEPATH=$1;
export TOP=$2;
export OUT_JSON=$(echo "$FILEPATH" | sed 's/\.v$/.json/' | sed 's/\/verilog\//\/json\//');

# yosys -p "verific -sv $FILEPATH" \
#       -p "verific -import -nosva $TOP" \
#       -p "proc" \
#       -p "memory" \
#       -p "opt_clean" \
#       -p "flatten" \
#       -p "opt_clean" \
#       -p "write_json $OUT_JSON"

## Is sometimes necessary to use read_verilog instead of verific
## E.g. on hackatdac18/soc_peripherals_netlist.v
yosys -p "read_verilog -sv $FILEPATH" \
      -p "hierarchy -top $TOP" \
      -p "proc" \
      -p "memory" \
      -p "opt_clean" \
      -p "flatten" \
      -p "opt_clean" \
      -p "write_json $OUT_JSON"