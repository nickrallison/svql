#!/bin/sh

rm -rf build;
cmake -DCMAKE_BUILD_TYPE=Debug -DCMAKE_C_COMPILER=gcc -DCMAKE_CXX_COMPILER=g++ -B build;
cmake --build build --parallel 32;

cargo build;