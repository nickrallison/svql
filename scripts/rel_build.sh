#!/bin/sh

rm -rf build;
cmake -DCMAKE_BUILD_TYPE=Release -B build;
cmake --build build --parallel 32;

cargo build --release;