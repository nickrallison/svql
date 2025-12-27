export FILEPATH="examples/fixtures/larger_designs/verilog/hackatdac21/openpiton_tile.v";
export TOP="tile";

export OUT_JSON=$(echo "$FILEPATH" | sed 's/\.v$/.json/' | sed 's/\/verilog\//\/json\//');

yosys -p "verific -sv \"$FILEPATH\"" -p "hierarchy -top $TOP" -p proc -p memory -p opt_clean -p flatten -p opt_clean -p "write_json \"$OUT_JSON\""