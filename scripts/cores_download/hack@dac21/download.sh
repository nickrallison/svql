#!/bin/sh

mkdir -p generated/

# if path not exist
if [ ! -d "generated/hackatdac21" ]; then
    git clone https://github.com/HACK-EVENT/hackatdac21.git generated/hackatdac21
fi

cd generated/hackatdac21
export PITON_ROOT=$(pwd)
export DV_ROOT=$(pwd)/piton

# source $PITON_ROOT/piton/piton_settings.bash
source $PITON_ROOT/piton/ariane_setup.sh
source $PITON_ROOT/piton/ariane_build_tools.sh

chmod +x $PITON_ROOT/piton/tools/bin/pyhp.py