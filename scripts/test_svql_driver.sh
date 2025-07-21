#!/usr/bin/env bash
set -euo pipefail

./yosys/yosys -m ./build/svql_driver/libsvql_driver.so scripts/svql_driver.ys