#!/bin/sh

cmake -B build
cmake --build build --target svql_driver
cd build
ctest -R svql_driver --output-on-failure