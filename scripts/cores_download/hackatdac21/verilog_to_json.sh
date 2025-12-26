export FILEPATH="examples/fixtures/larger_designs/verilog/hackatdac18/udma_core_netlist.v"; 

export OUT_JSON=$(echo "$FILEPATH" | sed 's/\.v$/.json/' | sed 's/\/verilog\//\/json\//');

yosys -p "read_verilog -sv \"$FILEPATH\"; proc; write_json \"$OUT_JSON\""
