#!/usr/bin/env sh
set -e

./yosys/yosys -m ./build/svql_driver/libsvql_driver.so scripts/test_net_driver.ys
