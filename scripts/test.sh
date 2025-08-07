#!/usr/bin/env sh
set -e

./yosys/yosys -m ./build/svql_driver/libsvql_driver.so scripts/svql_driver.ys &;

./target/debug/svql_driver
