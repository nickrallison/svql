#!/bin/sh

# Process Verilog files into RTLIL using Yosys
run_yosys() {
    TOP="$1"
    BASE_PATH="examples/patterns/basic/ff"
    
    if [ -z "$TOP" ]; then
        echo "Error: No top module name provided"
        exit 1
    fi

    yosys -p "read_verilog ${BASE_PATH}/verilog/${TOP}.v" \
          -p "hierarchy -top ${TOP}" \
          -p "proc" \
          -p "opt" \
          -p "clean" \
          -p "write_rtlil ${BASE_PATH}/rtlil/${TOP}.il"
}

    

# Usage example
run_yosys "adff"
run_yosys "adffe"
run_yosys "dff"
run_yosys "dffe"
run_yosys "sdff"
run_yosys "sdffe"