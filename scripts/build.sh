#!/bin/sh

cmake -B build; 
cmake --build build --parallel 32
cmake --build build --target svql_common_build --verbose