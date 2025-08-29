#!/bin/sh

mkdir -p generated/

git clone https://github.com/HACK-EVENT/hackatdac21.git generated/hackatdac21
cd generated/hackatdac21
export PITON_ROOT=$(pwd)

# source $PITON_ROOT/piton/piton_settings.bash
source $PITON_ROOT/piton/ariane_setup.sh
source $PITON_ROOT/piton/ariane_build_tools.sh