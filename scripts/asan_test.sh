#!/bin/sh

# export ENABLE_DEBUG=1; cd yosys; make clean; make -j 32 ENABLE_DEBUG=1;

cmake -DCMAKE_BUILD_TYPE=Debug -D ASAN_ENABLED=ON -B build
cmake --build build --parallel 32;

export LD_PRELOAD="/usr/lib/x86_64-linux-gnu/libasan.so.8"

./yosys/yosys -m ./build/svql_driver/libsvql_driver.so -p "read_verilog examples/cwe1234/locked_register_pat.v" -p "hierarchy -top locked_register_example" -p "proc" -p "svql_driver -pat svql_query/verilog/and.v and_gate -verbose"