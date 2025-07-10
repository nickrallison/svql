#!/bin/sh

cmake -B build; 
cmake --build build --parallel 32
