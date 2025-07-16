#!/bin/sh

# export ENABLE_DEBUG=1; cd yosys; make clean; make -j 32 ENABLE_DEBUG=1;

cmake -DCMAKE_BUILD_TYPE=Debug -B build;
cmake --build build --parallel 32;
