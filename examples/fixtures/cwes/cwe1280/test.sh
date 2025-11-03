
mkdir -p obj_dir

# Test vulnerable version
verilator --binary -j 0 examples/fixtures/cwes/cwe1280/verilog/cwe1280_vuln.v examples/fixtures/cwes/cwe1280/tb/cwe1280_vuln_tb.v --top-module cwe1280_vuln_tb
./obj_dir/Vcwe1280_vuln_tb

# Test fixed version
verilator --binary -j 0 examples/fixtures/cwes/cwe1280/verilog/cwe1280_fixed.v examples/fixtures/cwes/cwe1280/tb/cwe1280_fixed_tb.v --top-module cwe1280_fixed_tb
./obj_dir/Vcwe1280_fixed_tb