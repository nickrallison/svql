#!/bin/sh

cmake -DCMAKE_BUILD_TYPE=Debug -B build;
cmake --build build --parallel 32;
