#!/bin/sh

set -e

mkdir -p build
cmake -S . -B build -DYOSYS_BIN=$(which yosys) -DYOSYS_CONFIG=$(which yosys-config)
cmake --build build --parallel 32