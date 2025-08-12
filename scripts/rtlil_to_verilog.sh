# 1. Create a temporary directory for the generated Verilog files
mkdir -p generated_verilog

# Directory containing the RTLIL files
DIR="examples/patterns/security/access_control/locked_reg/rtlil"

# 2. Convert each RTLIL file to Verilog
for f in $DIR/*.il; do
    base=$(basename "$f" .il)          # e.g. async_en
    yosys -p "read_ilang $f; write_verilog -noattr generated_verilog/${base}.v"
done