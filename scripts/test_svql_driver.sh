      
#!/usr/bin/env bash
set -euo pipefail

# Resolve run-files. The path is <workspace_name>/<package_path>/<target_name>
# Your workspace name is 'svql' (from MODULE.bazel).
yosys_bin=$(rlocation svql/yosys/yosys)
plugin=$(rlocation svql/svql_driver/svql.so)
variant=$(rlocation svql/svql_driver/examples/cwe1234/variant1.v)
pattern=$(rlocation svql/svql_driver/examples/cwe1234/locked_register_pat.v)

# Run exactly the same Yosys flow as before ----------------------------------
"$yosys_bin" \
  -m "$plugin" \
  "$variant" \
  -p "hierarchy -top locked_register_example" \
  -p "proc" \
  -p "svql -map $pattern -verbose"
