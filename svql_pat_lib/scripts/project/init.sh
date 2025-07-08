#!/bin/sh

set -e

git submodule update --init --recursive;
cd libs/yosys;
make -j$(nproc);